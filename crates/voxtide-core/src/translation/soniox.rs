use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, Message},
};

use crate::audio::{CHANNELS, SAMPLE_RATE_HZ};
use crate::translation::tokens::{parse_message, ServerMessage, TranslationStatus};
use crate::translation::{FinalToken, Mode, SessionConfig, TranslationEvent, TranslationProvider};
use crate::{Error, Result};

pub const SONIOX_WS: &str = "wss://stt-rt.soniox.com/transcribe-websocket";
pub const MODEL: &str = "stt-rt-v4";

/// Bytes per millisecond of the audio we stream: pcm_s16le (2 bytes/sample)
/// at the pipeline's rate/channel constants = 16000 × 1 × 2 / 1000 = 32.
const PCM_BYTES_PER_MS: u64 = (SAMPLE_RATE_HZ as u64 * CHANNELS as u64 * 2) / 1000;

pub fn build_initial_config(cfg: &SessionConfig) -> Value {
    let mut base = json!({
        "api_key": cfg.api_key,
        "model": MODEL,
        // "pcm_s16le" is Soniox's wire name for the format; the rate/channel
        // values are the audio pipeline's own constants so the config can
        // never desync from what the capture path actually produces.
        "audio_format": "pcm_s16le",
        "sample_rate": SAMPLE_RATE_HZ,
        "num_channels": CHANNELS,
        "enable_endpoint_detection": true,
        "enable_speaker_diarization": true,
    });

    match cfg.mode {
        Mode::Meeting => {
            // language_a is the spoken (source) language; language_b is the
            // translation target. One-way a → b.
            base["language_hints"] = json!([cfg.language_a]);
            base["translation"] = json!({
                "type": "one_way",
                "target_language": cfg.language_b,
            });
        }
        Mode::Conversation => {
            base["translation"] = json!({
                "type": "two_way",
                "language_a": cfg.language_a,
                "language_b": cfg.language_b,
            });
        }
    }
    base
}

pub const MAX_ATTEMPTS: u32 = 6;

pub fn next_backoff_ms(attempt: u32) -> u64 {
    // attempt is 1-indexed.
    let raw = match attempt {
        1 => 250,
        2 => 500,
        3 => 1000,
        4 => 2000,
        _ => 5000,
    };
    raw.min(5000)
}

/// Internal control envelope between the public API and the background task.
enum Outbound {
    Binary(Vec<u8>),
}

/// Bring-your-own-key Soniox real-time translation provider.
///
/// Owns a background tokio task that holds the WebSocket connection and pumps
/// audio frames out / token events in. Reconnects on transient failures up to
/// [`MAX_ATTEMPTS`] using [`next_backoff_ms`].
pub struct SonioxBYOK {
    endpoint: String,
    /// Reconnect backoff schedule, injectable so tests can collapse the
    /// ~13.8s real-time ladder to near-zero. Defaults to [`next_backoff_ms`].
    backoff_ms: fn(u32) -> u64,
    audio_tx: Option<mpsc::Sender<Outbound>>,
    event_rx: Option<mpsc::Receiver<TranslationEvent>>,
    task: Option<tokio::task::JoinHandle<()>>,
}

impl SonioxBYOK {
    /// Construct a provider pointing at the production Soniox WebSocket.
    pub fn new() -> Self {
        Self::with_endpoint(SONIOX_WS)
    }

    /// Construct a provider with a custom endpoint (for tests or proxies).
    pub fn with_endpoint(endpoint: &str) -> Self {
        Self::with_endpoint_and_backoff(endpoint, next_backoff_ms)
    }

    /// Construct a provider with a custom endpoint AND a custom backoff
    /// schedule. Tests pass e.g. `|_| 1` to make reconnect attempts effectively
    /// instant so a self-termination assertion doesn't sleep the real ladder.
    pub fn with_endpoint_and_backoff(endpoint: &str, backoff_ms: fn(u32) -> u64) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            backoff_ms,
            audio_tx: None,
            event_rx: None,
            task: None,
        }
    }
}

impl Default for SonioxBYOK {
    fn default() -> Self {
        Self::new()
    }
}

/// Surface a terminal failure to the caller: emit [`TranslationEvent::Error`]
/// carrying the human-readable message, then [`TranslationEvent::Stopped`] to
/// end the session normally. Also traces the error. Kept as a helper so the
/// background task can fail-fast without code dup. Best-effort sends — if the
/// receiver is already gone the task is being torn down anyway.
async fn fail_with(event_tx: &mpsc::Sender<TranslationEvent>, msg: String) {
    tracing::error!(error = %msg, "SonioxBYOK stopping");
    let _ = event_tx.send(TranslationEvent::Error(msg)).await;
    let _ = event_tx.send(TranslationEvent::Stopped).await;
}

/// Emit the finals accumulated from the current frame as ONE
/// [`TranslationEvent::Finals`] event (no-op when empty). Called after the
/// frame's token loop, and additionally just before an `<end>` marker so an
/// utterance break never jumps ahead of finals that preceded it on the wire.
async fn flush_finals(
    event_tx: &mpsc::Sender<TranslationEvent>,
    finals: &mut Vec<FinalToken>,
    lag_ms: Option<u64>,
) {
    if finals.is_empty() {
        return;
    }
    let _ = event_tx
        .send(TranslationEvent::Finals {
            tokens: std::mem::take(finals),
            lag_ms,
        })
        .await;
}

#[async_trait::async_trait]
impl TranslationProvider for SonioxBYOK {
    async fn open(&mut self, cfg: SessionConfig) -> Result<()> {
        let (audio_tx, mut audio_rx) = mpsc::channel::<Outbound>(64);
        let (event_tx, event_rx) = mpsc::channel::<TranslationEvent>(64);
        self.audio_tx = Some(audio_tx);
        self.event_rx = Some(event_rx);

        let endpoint = self.endpoint.clone();
        let backoff_ms = self.backoff_ms;

        let task = tokio::spawn(async move {
            let mut attempt = 0u32;
            // Reconnect-loop invariant: every back-edge re-checks
            // `attempt > MAX_ATTEMPTS` before sleeping/retrying, so an endpoint
            // that never makes progress always terminates. `attempt` resets to 0
            // only on real-token progress (`got_tokens`); terminal exits happen
            // only via `fail_with(...) + return` or a `Stopped`-emitting `return`.
            'outer: loop {
                attempt += 1;

                let req = match endpoint.as_str().into_client_request() {
                    Ok(r) => r,
                    Err(e) => {
                        fail_with(&event_tx, format!("invalid endpoint: {e}")).await;
                        return;
                    }
                };

                let ws = match connect_async(req).await {
                    Ok((ws, _)) => ws,
                    Err(e) => {
                        if attempt > MAX_ATTEMPTS {
                            fail_with(&event_tx, format!("connect: {e}")).await;
                            return;
                        }
                        let wait = backoff_ms(attempt);
                        let _ = event_tx
                            .send(TranslationEvent::Reconnecting {
                                attempt,
                                retry_in_ms: wait,
                            })
                            .await;
                        tokio::time::sleep(Duration::from_millis(wait)).await;
                        continue 'outer;
                    }
                };
                let _ = event_tx.send(TranslationEvent::Connected).await;

                let (mut ws_tx, mut ws_rx) = ws.split();
                let initial = build_initial_config(&cfg).to_string();
                if let Err(e) = ws_tx.send(Message::Text(initial)).await {
                    tracing::warn!(?e, "send initial config; reconnecting");
                    // A failed config send is NOT progress, so we do NOT reset
                    // `attempt`. Bound it like any other reconnect cause so an
                    // endpoint that accepts the handshake then refuses the
                    // config can't drive an unbounded loop.
                    if attempt > MAX_ATTEMPTS {
                        fail_with(
                            &event_tx,
                            format!(
                                "connection failed {attempt} times without progress; giving up"
                            ),
                        )
                        .await;
                        return;
                    }
                    let wait = backoff_ms(attempt);
                    let _ = event_tx
                        .send(TranslationEvent::Reconnecting {
                            attempt,
                            retry_in_ms: wait,
                        })
                        .await;
                    tokio::time::sleep(Duration::from_millis(wait)).await;
                    continue 'outer;
                }

                // Per-connection progress flag. Set true once we process a real
                // (non-control-marker) token this connection. Only real progress
                // resets the reconnect budget below; an accept-then-close server
                // (captive portal, proxy, idle-close, maintenance) never sets it,
                // so the loop terminates via `MAX_ATTEMPTS`.
                let mut got_tokens = false;
                let mut finished = false;
                let mut client_eos = false;
                // Milliseconds of audio successfully written to THIS
                // connection's socket. Per-connection (like Soniox's
                // `final_audio_proc_ms` watermark, which restarts at 0 on a
                // fresh connection) so the audio-anchored lag stays honest
                // across reconnects.
                let mut audio_ms_sent: u64 = 0;
                'inner: loop {
                    tokio::select! {
                        // Outbound audio from the caller (borrowed by recv, survives reconnect).
                        // Once the client has sent EOS we stop polling the audio side; otherwise
                        // a closed sender would race us into `break 'inner` and we'd miss the
                        // server's final tokens / `finished` marker.
                        out = audio_rx.recv(), if !client_eos => {
                            match out {
                                Some(Outbound::Binary(b)) => {
                                    let bytes = b.len() as u64;
                                    if let Err(e) = ws_tx.send(Message::Binary(b)).await {
                                        tracing::warn!(?e, "send audio; reconnecting");
                                        break 'inner;
                                    }
                                    // Successful write: advance the watermark the
                                    // audio-anchored latency is measured against.
                                    audio_ms_sent += bytes / PCM_BYTES_PER_MS;
                                }
                                None => {
                                    // Caller dropped the audio sender (i.e., called close()).
                                    // Treat as client EOS: forward to server and stop polling audio.
                                    let _ = ws_tx.send(Message::Text(String::new())).await;
                                    client_eos = true;
                                }
                            }
                        }
                        // Inbound from server.
                        msg = ws_rx.next() => {
                            match msg {
                                Some(Ok(m)) => {
                                    if m.is_close() { break 'inner; }
                                    if let Message::Text(s) = m {
                                        match parse_message(&s) {
                                            Ok(ServerMessage::Tokens(frame)) => {
                                                // Soniox sends non-final tokens as a complete trailing-partial
                                                // replacement per frame. Batch the frame's finals into ONE
                                                // Finals event (per-token ts_ms + speaker preserved inside;
                                                // one event = one DB transaction downstream), and combine
                                                // non-finals into one Live event per (translation vs not)
                                                // bucket — frontend stores a single partial per status, and
                                                // emitting one per token would leave only the last sub-word
                                                // visible.
                                                //
                                                // Audio-anchored lag for this frame: how far the server's
                                                // fully-processed watermark trails the audio we've written.
                                                let lag_ms = frame
                                                    .final_audio_proc_ms
                                                    .map(|p| audio_ms_sent.saturating_sub(p));
                                                let mut finals: Vec<FinalToken> = Vec::new();
                                                let mut orig_text = String::new();
                                                let mut orig_meta: Option<(Option<String>, Option<String>)> = None;
                                                let mut trans_text = String::new();
                                                let mut trans_meta: Option<(Option<String>, Option<String>)> = None;
                                                for t in frame.tokens {
                                                    // Soniox emits control markers like `<end>` and `<fin>`
                                                    // (endpoint detection, silence) as ordinary-looking tokens
                                                    // wrapped in angle brackets. They are never user transcript
                                                    // text. `<end>` is the endpoint/pause signal (we enabled
                                                    // `enable_endpoint_detection`) — surface it as an utterance
                                                    // break so the store can chunk a long monologue. Other
                                                    // markers (`<fin>`, …) are skipped silently to avoid
                                                    // over-segmenting on every finalization.
                                                    if t.text.starts_with('<') && t.text.ends_with('>') {
                                                        if t.text == "<end>" {
                                                            // Flush finals seen so far FIRST so the break stays
                                                            // exactly where the wire put it relative to them.
                                                            flush_finals(&event_tx, &mut finals, lag_ms).await;
                                                            let _ = event_tx
                                                                .send(TranslationEvent::UtteranceBreak)
                                                                .await;
                                                        }
                                                        continue;
                                                    }
                                                    // A real transcript token: this connection made
                                                    // progress, so the reconnect budget may reset.
                                                    got_tokens = true;
                                                    if t.is_final {
                                                        finals.push(FinalToken {
                                                            text: t.text,
                                                            language: t.language,
                                                            status: t.translation_status,
                                                            speaker: t.speaker,
                                                            ts_ms: crate::now_ms(),
                                                        });
                                                    } else if matches!(t.translation_status, TranslationStatus::Translation) {
                                                        trans_text.push_str(&t.text);
                                                        trans_meta = Some((t.language, t.speaker));
                                                    } else {
                                                        orig_text.push_str(&t.text);
                                                        orig_meta = Some((t.language, t.speaker));
                                                    }
                                                }
                                                flush_finals(&event_tx, &mut finals, lag_ms).await;
                                                if let Some((language, speaker)) = orig_meta {
                                                    let _ = event_tx
                                                        .send(TranslationEvent::Live {
                                                            text: orig_text,
                                                            language,
                                                            status: TranslationStatus::Original,
                                                            speaker,
                                                        })
                                                        .await;
                                                }
                                                if let Some((language, speaker)) = trans_meta {
                                                    let _ = event_tx
                                                        .send(TranslationEvent::Live {
                                                            text: trans_text,
                                                            language,
                                                            status: TranslationStatus::Translation,
                                                            speaker,
                                                        })
                                                        .await;
                                                }
                                                if frame.finished { finished = true; break 'inner; }
                                            }
                                            Ok(ServerMessage::Finished) => {
                                                finished = true;
                                                break 'inner;
                                            }
                                            Ok(ServerMessage::Error { code, message }) => {
                                                // Best-effort close so the server sees a clean
                                                // shutdown before we tear the task down.
                                                let _ = ws_tx.close().await;
                                                fail_with(
                                                    &event_tx,
                                                    format!("Soniox error {code}: {message}"),
                                                )
                                                .await;
                                                return;
                                            }
                                            Err(e) => {
                                                tracing::warn!(?e, raw=%s, "parse");
                                            }
                                        }
                                    }
                                }
                                Some(Err(e)) => {
                                    tracing::warn!(?e, "recv error; reconnecting");
                                    break 'inner;
                                }
                                None => break 'inner,
                            }
                        }
                    }
                }

                let _ = ws_tx.close().await;

                if finished {
                    let _ = event_tx.send(TranslationEvent::Stopped).await;
                    return;
                }
                // If the caller already sent EOS, don't reconnect — they're done.
                if client_eos {
                    let _ = event_tx.send(TranslationEvent::Stopped).await;
                    return;
                }

                // The connection was established but died. Reset the reconnect
                // budget ONLY if this connection made real progress (received at
                // least one transcript token); otherwise the budget keeps
                // climbing. An endpoint that accepts the handshake then closes
                // without ever sending tokens (captive portal, proxy, Soniox
                // idle-close, maintenance) thus exhausts MAX_ATTEMPTS and the
                // session self-terminates instead of flapping forever.
                if got_tokens {
                    attempt = 0;
                }
                if attempt > MAX_ATTEMPTS {
                    fail_with(
                        &event_tx,
                        format!("connection failed {attempt} times without progress; giving up"),
                    )
                    .await;
                    return;
                }
                // `next_backoff_ms` is 1-indexed; `attempt` is the count of the
                // attempt we're about to retry from. `attempt` is 0 only right
                // after the progress reset above, so `max(1)` re-floors it to the
                // first rung — equivalent to the old `if got_tokens { 1 } else
                // { attempt }`. Note this deliberately re-emits `Reconnecting
                // { attempt: 1 }` after a productive connection dies: a fresh
                // reconnect ladder is the intended UX, not a bug.
                let next = attempt.max(1);
                let wait = backoff_ms(next);
                let _ = event_tx
                    .send(TranslationEvent::Reconnecting {
                        attempt: next,
                        retry_in_ms: wait,
                    })
                    .await;
                tokio::time::sleep(Duration::from_millis(wait)).await;
            }
        });

        self.task = Some(task);
        Ok(())
    }

    async fn send_audio(&mut self, pcm: Vec<u8>) -> Result<()> {
        let Some(tx) = self.audio_tx.as_ref() else {
            return Err(Error::Soniox("provider not open".into()));
        };
        // Move the buffer straight into the outbound channel — no extra copy.
        tx.send(Outbound::Binary(pcm))
            .await
            .map_err(|e| Error::Soniox(format!("audio channel closed: {e}")))
    }

    async fn next_event(&mut self) -> Option<TranslationEvent> {
        let rx = self.event_rx.as_mut()?;
        rx.recv().await
    }

    async fn eos(&mut self) {
        // Initiate the EOS handshake without tearing anything down. Dropping the
        // audio sender makes the background task observe `audio_rx.recv() -> None`
        // and forward an EOS sentinel to the server; the server then flushes its
        // trailing finals. We deliberately KEEP `event_rx` so those finals (the
        // last words spoken before stop) still reach the caller, who drains them
        // before calling `close()`. Idempotent: a second call (or one after
        // `close()`) just re-takes an already-None sender — a no-op.
        self.audio_tx = None;
    }

    async fn close(&mut self) -> Result<()> {
        // Dropping the sender closes the channel. The background task detects
        // `audio_rx.recv() -> None` and treats it as client EOS: it forwards an
        // EOS sentinel to the server and exits cleanly without needing abort().
        // Harmless if `eos()` already dropped it — this just re-takes None.
        self.audio_tx = None;

        let result = if let Some(mut task) = self.task.take() {
            // `&mut task` is itself a future (JoinHandle: Unpin), so the timeout
            // borrows the handle rather than consuming it. On Err we still own
            // `task` and abort it — otherwise the background task keeps running
            // parked on `ws_rx.next()`, holding the TLS socket + channels for
            // the rest of the process lifetime (one leaked task per stuck close).
            match tokio::time::timeout(Duration::from_secs(3), &mut task).await {
                Ok(Ok(())) => Ok(()),
                Ok(Err(join_err)) => Err(Error::Soniox(format!("join: {join_err}"))),
                Err(_) => {
                    // Fire-and-forget: abort() only requests cancellation. The
                    // teardown (socket + channel drop) completes on the cancelled
                    // task's next poll at an await point; we deliberately do NOT
                    // wait for it here — close() must not re-block past its budget.
                    task.abort();
                    Err(Error::Soniox("close timed out".into()))
                }
            }
        } else {
            Ok(())
        };

        // Always clear the event channel so next_event() returns None immediately.
        self.event_rx = None;
        result
    }
}
