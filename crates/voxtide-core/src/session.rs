//! Session orchestration. Wires [`AudioSource`] + [`TranslationProvider`] + [`Store`] together,
//! owns the per-session speaker map and latency tracker, and emits [`CoreEvent`] on a broadcast
//! channel that the Tauri layer (Plan 2) subscribes to.
//!
//! The controller spawns a single task per session that owns the provider outright and uses
//! [`tokio::select!`] to multiplex outbound audio chunks with inbound translation events. This
//! avoids the latency penalty of a shared `Mutex<Provider>` (where `next_event().await` would
//! starve `send_audio`).

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::sync::broadcast;

use crate::audio::AudioSource;
use crate::latency::LatencyTracker;
use crate::persistence::sessions::{NewSession, Sessions};
use crate::persistence::tokens::{NewToken, Tokens};
use crate::persistence::Store;
use crate::speaker_map::SpeakerMap;
use crate::translation::tokens::TranslationStatus;
use crate::translation::{Mode, SessionConfig, TranslationEvent, TranslationProvider};
use crate::Result;

/// Events broadcast by the [`SessionController`] for UI / orchestration consumers.
#[derive(Debug, Clone)]
pub enum CoreEvent {
    SessionStarted {
        session_id: i64,
        mode: String,
    },
    SessionStopped {
        session_id: i64,
        duration_ms: i64,
    },
    TranscriptLive {
        status: TranslationStatus,
        text: String,
        language: Option<String>,
        chip: Option<char>,
    },
    TranscriptFinal {
        status: TranslationStatus,
        text: String,
        language: Option<String>,
        chip: Option<char>,
        ts_ms: i64,
    },
    /// Speech pause delimiting utterances; the UI starts a new row.
    UtteranceBreak,
    ConnectionState {
        state: &'static str,
        attempt: Option<u32>,
        retry_in_ms: Option<u64>,
    },
    Latency {
        median_ms: u64,
    },
}

/// Inputs required to start a session.
pub struct StartArgs {
    pub cfg: SessionConfig,
    pub source: Box<dyn AudioSource>,
    pub provider: Box<dyn TranslationProvider>,
    pub device_label: Option<String>,
}

/// Tri-state slot that prevents TOCTOU races in [`SessionController::start`].
///
/// The transition is always `Idle → Pending → Running` (on success) or
/// `Pending → Idle` (on any error between the two). A second concurrent
/// `start()` sees `Pending` and returns `Err` immediately, so no race is
/// possible even across `await` points.
enum RunState {
    Idle,
    /// `start()` is in progress; async setup is executing.
    Pending,
    /// A session is live. Holds the handles needed to stop it.
    Running(RunningSession),
}

/// Orchestrates one in-flight session at a time. Cheap to clone the `Arc<Store>`; the controller
/// itself is owned by the Tauri app state.
pub struct SessionController {
    store: Arc<Store>,
    tx: broadcast::Sender<CoreEvent>,
    running: Arc<parking_lot::Mutex<RunState>>,
}

struct RunningSession {
    session_id: i64,
    join: tokio::task::JoinHandle<()>,
    stop_audio: tokio::sync::oneshot::Sender<()>,
    /// Signals the worker loop to break, persist the session end, and close the provider. We use
    /// a separate channel from `stop_audio` because closing the audio source alone leaves the
    /// worker blocked on `provider.next_event()`.
    stop_worker: tokio::sync::oneshot::Sender<()>,
}

/// RAII guard that resets the [`RunState`] slot back to [`RunState::Idle`] if
/// `start()` returns early via `?` between setting the slot to `Pending` and
/// completing the full setup. Disarmed on success by setting `armed = false`
/// before the guard goes out of scope.
struct StartGuard {
    slot: Arc<parking_lot::Mutex<RunState>>,
    armed: bool,
}

impl Drop for StartGuard {
    fn drop(&mut self) {
        if self.armed {
            *self.slot.lock() = RunState::Idle;
        }
    }
}

impl SessionController {
    pub fn new(store: Store) -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            store: Arc::new(store),
            tx,
            running: Arc::new(parking_lot::Mutex::new(RunState::Idle)),
        }
    }

    /// New subscribers receive events sent after [`broadcast::Sender::subscribe`] returns. Lagging
    /// receivers see [`broadcast::error::RecvError::Lagged`] and skip ahead — the UI must treat
    /// this as a refresh signal rather than a fatal error.
    pub fn subscribe(&self) -> broadcast::Receiver<CoreEvent> {
        self.tx.subscribe()
    }

    pub fn store(&self) -> &Store {
        &self.store
    }

    /// Returns the id of the currently-running session, or `None` when idle or pending.
    /// `Pending` deliberately reports `None` — the id is unknown until the session row has
    /// been created and the slot has flipped to `Running`.
    pub fn active_session_id(&self) -> Option<i64> {
        match &*self.running.lock() {
            RunState::Running(r) => Some(r.session_id),
            _ => None,
        }
    }

    pub async fn start(&self, args: StartArgs) -> Result<i64> {
        // Atomically claim the slot before any await point. If another start() is already in
        // progress (Pending) or a session is running (Running), we reject immediately. Setting
        // Pending here prevents a second concurrent future that also passes this check — both
        // Pending and Running are treated as "already running".
        {
            let mut slot = self.running.lock();
            match *slot {
                RunState::Idle => *slot = RunState::Pending,
                _ => return Err(crate::Error::Session("already running".into())),
            }
        }
        // If any `?` below triggers, `guard` will reset the slot back to Idle on drop.
        let mut guard = StartGuard {
            slot: Arc::clone(&self.running),
            armed: true,
        };

        let started_at = now_ms();
        let mode_str = match args.cfg.mode {
            Mode::Meeting => "meeting",
            Mode::Conversation => "conversation",
        };
        // Capture the fields needed for the row before `provider.open` consumes `args.cfg`.
        let lang_a = args.cfg.language_a.clone();
        let lang_b = args.cfg.language_b.clone();
        let device_label = args.device_label.clone();

        // Perform every fallible setup step BEFORE creating the DB row or broadcasting
        // `SessionStarted`. If any of these fail, the controller is left exactly as it was
        // (the StartGuard resets the slot to Idle on drop): no orphan `ended_at IS NULL` row,
        // and no latched `recording=true` on the frontend. Ordering: open provider → start
        // source → create row → broadcast → spawn worker.

        // Open the provider on the caller's task so that authentication / handshake errors surface
        // synchronously to `start()` rather than vanishing inside the spawned worker.
        let mut provider = args.provider;
        provider.open(args.cfg).await?;

        // Start the audio source. On failure, close the provider best-effort (it was just opened)
        // and propagate the error. Nothing irreversible has happened yet.
        let stream = match args.source.start() {
            Ok(s) => s,
            Err(e) => {
                let _ = provider.close().await;
                return Err(e);
            }
        };
        let mut audio_rx = stream.rx;
        let stop_audio = stream.stop;

        // Persist the session row only once the live machinery is confirmed up. This guarantees we
        // have a stable id to attach tokens to. If row creation fails, tear down everything we
        // started (drop the stream to stop the source, close the provider) and propagate — leaving
        // no half-started session behind.
        let session_id = match Sessions::create(
            self.store.pool(),
            NewSession {
                started_at,
                mode: mode_str.into(),
                lang_a,
                lang_b,
                device_label,
            },
        )
        .await
        {
            Ok(id) => id,
            Err(e) => {
                // Dropping the stream's `stop` sender signals the source thread/task to halt.
                drop(stop_audio);
                drop(audio_rx);
                let _ = provider.close().await;
                return Err(e);
            }
        };
        // Nothing fallible remains after the row exists; the SessionStarted broadcast and worker
        // spawn below cannot fail, so the frontend's latched `recording=true` is always backed by
        // a real, finalizable session.
        let _ = self.tx.send(CoreEvent::SessionStarted {
            session_id,
            mode: mode_str.into(),
        });

        let store = self.store.clone();
        let tx = self.tx.clone();
        let running = Arc::clone(&self.running);

        let (stop_worker_tx, mut stop_worker_rx) = tokio::sync::oneshot::channel::<()>();

        let join = tokio::spawn(async move {
            let mut speakers = SpeakerMap::new();
            let mut latency = LatencyTracker::new(64);
            let started = started_at;
            // Once the audio source is drained we still need to keep polling the provider for the
            // remaining tokens and the terminal Stopped event. `audio_done` disables the audio arm
            // via a `tokio::select!` precondition so we don't busy-spin on a closed channel.
            let mut audio_done = false;

            loop {
                tokio::select! {
                    // Explicit stop request from `SessionController::stop()`. Persist the end of
                    // the session, broadcast the stopped event, and break out of the loop. We
                    // never want a stale audio frame or a delayed provider event to delay this.
                    biased;
                    _ = &mut stop_worker_rx => {
                        // Finalization is centralized after the loop so every
                        // exit path persists the session exactly once.
                        break;
                    }
                    // Forward audio chunks to the provider. Disabled once the source is drained.
                    maybe_frame = audio_rx.recv(), if !audio_done => {
                        match maybe_frame {
                            Some(frame) => {
                                let bytes = frame.to_le_bytes();
                                if let Err(e) = provider.send_audio(&bytes).await {
                                    tracing::warn!(?e, "provider send_audio");
                                    let _ = tx.send(CoreEvent::ConnectionState {
                                        state: "reconnecting",
                                        attempt: Some(1),
                                        retry_in_ms: Some(250),
                                    });
                                }
                            }
                            None => {
                                audio_done = true;
                            }
                        }
                    }
                    // Receive translation events. Terminates the loop on None (closed) or Stopped.
                    ev = provider.next_event() => {
                        let Some(ev) = ev else { break; };
                        match ev {
                            TranslationEvent::Connected => {
                                let _ = tx.send(CoreEvent::ConnectionState {
                                    state: "active",
                                    attempt: None,
                                    retry_in_ms: None,
                                });
                            }
                            TranslationEvent::Reconnecting { attempt, retry_in_ms } => {
                                let _ = tx.send(CoreEvent::ConnectionState {
                                    state: "reconnecting",
                                    attempt: Some(attempt),
                                    retry_in_ms: Some(retry_in_ms),
                                });
                            }
                            TranslationEvent::Live { text, language, status, speaker } => {
                                let chip = speaker.as_deref().map(|s| speakers.chip_for(s));
                                let _ = tx.send(CoreEvent::TranscriptLive {
                                    status,
                                    text,
                                    language,
                                    chip,
                                });
                            }
                            TranslationEvent::Final { text, language, status, speaker, ts_ms } => {
                                let chip = speaker.as_deref().map(|s| speakers.chip_for(s));
                                let speaker_letter = chip.map(|c| c.to_string());
                                // Persist before broadcasting so a downstream listener that
                                // immediately queries the store sees the row.
                                if let Err(e) = Tokens::insert(
                                    store.pool(),
                                    NewToken {
                                        session_id,
                                        ts_ms: ts_ms - started,
                                        text: text.clone(),
                                        language: language.clone(),
                                        status: match status {
                                            TranslationStatus::Translation => "translation".into(),
                                            _ => "original".into(),
                                        },
                                        speaker: speaker_letter,
                                    },
                                )
                                .await
                                {
                                    tracing::warn!(?e, "tokens insert");
                                }
                                // Soniox emits `ts_ms` as a stream-relative offset, so the
                                // wall-clock subtraction here is only meaningful for the live
                                // path. We saturate at zero to avoid wrap-around noise on the
                                // mock provider, which can report timestamps in the past.
                                latency.observe((now_ms() - ts_ms).max(0) as u64);
                                if let Some(m) = latency.median_ms() {
                                    let _ = tx.send(CoreEvent::Latency { median_ms: m });
                                }
                                let _ = tx.send(CoreEvent::TranscriptFinal {
                                    status,
                                    text,
                                    language,
                                    chip,
                                    ts_ms,
                                });
                            }
                            TranslationEvent::UtteranceBreak => {
                                let _ = tx.send(CoreEvent::UtteranceBreak);
                            }
                            TranslationEvent::Stopped => break,
                        }
                    }
                }
            }

            // Best-effort cleanup. If the provider already closed itself (Stopped path) this is a
            // no-op; if the loop broke for another reason we want the socket released promptly.
            let _ = provider.close().await;

            // Finalize on EVERY loop-exit path: explicit stop(), provider
            // `Stopped`, or the provider stream closing with no terminal event
            // (websocket drop / auth expiry). Persisting here rather than in
            // the individual select arms guarantees a session can never be
            // left stuck `ended_at IS NULL` while the process keeps running.
            // `Sessions::finish` is an idempotent UPDATE, so a redundant call
            // is harmless.
            let ended = now_ms();
            if let Err(e) = Sessions::finish(store.pool(), session_id, ended).await {
                tracing::warn!(?e, "sessions finish (post-loop)");
            }
            let _ = tx.send(CoreEvent::SessionStopped {
                session_id,
                duration_ms: ended - started,
            });

            // Reset the slot to Idle so callers (e.g. active_session_id) see the correct state
            // after a natural stop. Explicit stop() already resets to Idle before joining, so we
            // only update here when the slot still holds OUR session. The identity check is
            // critical: a worker that outlived stop()'s 5 s join timeout (e.g. parked in
            // provider.send_audio during a network blackhole) must NOT stomp a *newer* session's
            // slot — doing so would drop the new session's RunningSession, firing its stop_worker
            // oneshot and killing session B seconds after it started.
            let mut slot = running.lock();
            if matches!(&*slot, RunState::Running(r) if r.session_id == session_id) {
                *slot = RunState::Idle;
            }
        });

        *self.running.lock() = RunState::Running(RunningSession {
            session_id,
            join,
            stop_audio,
            stop_worker: stop_worker_tx,
        });
        // Disarm: setup succeeded, do not reset the slot on drop.
        guard.armed = false;
        Ok(session_id)
    }

    /// Stop the currently-running session. Signals the audio source to stop and the worker loop
    /// to exit, then waits up to 5 s for the worker task to finish. If it overruns we drop the
    /// handle and let it complete in the background rather than blocking the caller indefinitely.
    pub async fn stop(&self) -> Result<()> {
        // Take the Running variant and reset to Idle in one lock operation.
        let maybe_running = {
            let mut slot = self.running.lock();
            match std::mem::replace(&mut *slot, RunState::Idle) {
                RunState::Running(r) => Some(r),
                other => {
                    // Restore whatever state we found (Idle or Pending) — we didn't take anything.
                    *slot = other;
                    None
                }
            }
        };
        if let Some(r) = maybe_running {
            let _ = r.stop_audio.send(());
            let _ = r.stop_worker.send(());
            let _ = tokio::time::timeout(std::time::Duration::from_secs(5), r.join).await;
        }
        Ok(())
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
