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

/// Orchestrates one in-flight session at a time. Cheap to clone the `Arc<Store>`; the controller
/// itself is owned by the Tauri app state.
pub struct SessionController {
    store: Arc<Store>,
    tx: broadcast::Sender<CoreEvent>,
    running: parking_lot::Mutex<Option<RunningSession>>,
}

struct RunningSession {
    join: tokio::task::JoinHandle<()>,
    stop_audio: tokio::sync::oneshot::Sender<()>,
    /// Signals the worker loop to break, persist the session end, and close the provider. We use
    /// a separate channel from `stop_audio` because closing the audio source alone leaves the
    /// worker blocked on `provider.next_event()`.
    stop_worker: tokio::sync::oneshot::Sender<()>,
}

impl SessionController {
    pub fn new(store: Store) -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            store: Arc::new(store),
            tx,
            running: parking_lot::Mutex::new(None),
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

    pub async fn start(&self, args: StartArgs) -> Result<i64> {
        // Double-start guard: refuse if a session is already running. Without this, repeated
        // calls would leak the prior `RunningSession`'s join handle and broadcast spurious
        // SessionStarted events.
        if self.running.lock().is_some() {
            return Err(crate::Error::Session("already running".into()));
        }

        let started_at = now_ms();
        let mode_str = match args.cfg.mode {
            Mode::Meeting => "meeting",
            Mode::Conversation => "conversation",
        };

        // Persist the session row before kicking off the live machinery; this guarantees we have a
        // stable id to attach tokens to even if the provider fails on the very first event.
        let session_id = Sessions::create(
            self.store.pool(),
            NewSession {
                started_at,
                mode: mode_str.into(),
                lang_a: args.cfg.language_a.clone(),
                lang_b: args.cfg.language_b.clone(),
                device_label: args.device_label.clone(),
            },
        )
        .await?;
        let _ = self.tx.send(CoreEvent::SessionStarted {
            session_id,
            mode: mode_str.into(),
        });

        let store = self.store.clone();
        let tx = self.tx.clone();

        // Open the provider on the caller's task so that authentication / handshake errors surface
        // synchronously to `start()` rather than vanishing inside the spawned worker.
        let mut provider = args.provider;
        provider.open(args.cfg).await?;

        let stream = args.source.start()?;
        let mut audio_rx = stream.rx;
        let stop_audio = stream.stop;
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
                        let ended = now_ms();
                        if let Err(e) = Sessions::finish(store.pool(), session_id, ended).await {
                            tracing::warn!(?e, "sessions finish (on stop)");
                        }
                        let _ = tx.send(CoreEvent::SessionStopped {
                            session_id,
                            duration_ms: ended - started,
                        });
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
                            TranslationEvent::Stopped => {
                                let ended = now_ms();
                                if let Err(e) = Sessions::finish(store.pool(), session_id, ended).await {
                                    tracing::warn!(?e, "sessions finish");
                                }
                                let _ = tx.send(CoreEvent::SessionStopped {
                                    session_id,
                                    duration_ms: ended - started,
                                });
                                break;
                            }
                        }
                    }
                }
            }

            // Best-effort cleanup. If the provider already closed itself (Stopped path) this is a
            // no-op; if the loop broke for another reason we want the socket released promptly.
            let _ = provider.close().await;
        });

        *self.running.lock() = Some(RunningSession {
            join,
            stop_audio,
            stop_worker: stop_worker_tx,
        });
        Ok(session_id)
    }

    /// Stop the currently-running session. Signals the audio source to stop and the worker loop
    /// to exit, then waits up to 5 s for the worker task to finish. If it overruns we drop the
    /// handle and let it complete in the background rather than blocking the caller indefinitely.
    pub async fn stop(&self) -> Result<()> {
        let running = self.running.lock().take();
        if let Some(r) = running {
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
