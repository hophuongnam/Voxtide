use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, Message},
};

use crate::translation::tokens::{parse_message, ServerMessage, TranslationStatus};
use crate::translation::{Mode, SessionConfig, TranslationEvent, TranslationProvider, WhichLang};
use crate::{Error, Result};

pub const SONIOX_WS: &str = "wss://stt-rt.soniox.com/transcribe-websocket";
pub const MODEL: &str = "stt-rt-v4";

pub fn build_initial_config(cfg: &SessionConfig) -> Value {
    let mut base = json!({
        "api_key": cfg.api_key,
        "model": MODEL,
        "audio_format": "pcm_s16le",
        "sample_rate": 16000,
        "num_channels": 1,
        "enable_endpoint_detection": true,
        "enable_speaker_diarization": true,
    });

    let (mine_lang, other_lang) = match cfg.mine {
        WhichLang::A => (cfg.language_a.clone(), cfg.language_b.clone()),
        WhichLang::B => (cfg.language_b.clone(), cfg.language_a.clone()),
    };

    match cfg.mode {
        Mode::Meeting => {
            base["language_hints"] = json!([other_lang]);
            base["translation"] = json!({
                "type": "one_way",
                "target_language": mine_lang,
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
        Self {
            endpoint: endpoint.to_string(),
            audio_tx: None,
            event_rx: None,
            task: None,
        }
    }

    /// Wall-clock milliseconds since the Unix epoch. Saturates to 0 on clock
    /// skew (pre-1970) so we never panic.
    fn now_ms() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    }
}

impl Default for SonioxBYOK {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a terminal [`TranslationEvent::Stopped`] preceded by an error trace.
/// Kept as a helper so the background task can fail-fast without code dup.
fn stopped_with_err(msg: String) -> TranslationEvent {
    tracing::error!(error = %msg, "SonioxBYOK stopping");
    TranslationEvent::Stopped
}

#[async_trait::async_trait]
impl TranslationProvider for SonioxBYOK {
    async fn open(&mut self, cfg: SessionConfig) -> Result<()> {
        let (audio_tx, mut audio_rx) = mpsc::channel::<Outbound>(64);
        let (event_tx, event_rx) = mpsc::channel::<TranslationEvent>(64);
        self.audio_tx = Some(audio_tx);
        self.event_rx = Some(event_rx);

        let endpoint = self.endpoint.clone();

        let task = tokio::spawn(async move {
            let mut attempt = 0u32;
            'outer: loop {
                attempt += 1;

                let req = match endpoint.as_str().into_client_request() {
                    Ok(r) => r,
                    Err(e) => {
                        let _ = event_tx
                            .send(stopped_with_err(format!("invalid endpoint: {e}")))
                            .await;
                        return;
                    }
                };

                let ws = match connect_async(req).await {
                    Ok((ws, _)) => ws,
                    Err(e) => {
                        if attempt > MAX_ATTEMPTS {
                            let _ = event_tx
                                .send(stopped_with_err(format!("connect: {e}")))
                                .await;
                            return;
                        }
                        let wait = next_backoff_ms(attempt);
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
                    attempt = 1;
                    let wait = next_backoff_ms(attempt);
                    let _ = event_tx
                        .send(TranslationEvent::Reconnecting {
                            attempt,
                            retry_in_ms: wait,
                        })
                        .await;
                    tokio::time::sleep(Duration::from_millis(wait)).await;
                    continue 'outer;
                }

                let mut finished = false;
                let mut client_eos = false;
                'inner: loop {
                    tokio::select! {
                        // Outbound audio from the caller (borrowed by recv, survives reconnect).
                        // Once the client has sent EOS we stop polling the audio side; otherwise
                        // a closed sender would race us into `break 'inner` and we'd miss the
                        // server's final tokens / `finished` marker.
                        out = audio_rx.recv(), if !client_eos => {
                            match out {
                                Some(Outbound::Binary(b)) => {
                                    if let Err(e) = ws_tx.send(Message::Binary(b)).await {
                                        tracing::warn!(?e, "send audio; reconnecting");
                                        break 'inner;
                                    }
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
                                                // replacement per frame. Emit finals individually (preserves
                                                // per-token ts_ms + speaker grouping), but combine each frame's
                                                // non-finals into one Live event per (translation vs not)
                                                // bucket — frontend stores a single partial per status, and
                                                // emitting one per token would leave only the last sub-word
                                                // visible.
                                                let mut orig_text = String::new();
                                                let mut orig_meta: Option<(Option<String>, Option<String>)> = None;
                                                let mut trans_text = String::new();
                                                let mut trans_meta: Option<(Option<String>, Option<String>)> = None;
                                                for t in frame.tokens {
                                                    // Soniox emits control markers like `<end>` and `<fin>`
                                                    // (endpoint detection, silence) as ordinary-looking tokens
                                                    // wrapped in angle brackets. Skip them — they aren't user
                                                    // transcript text. (Currently we don't use them as
                                                    // utterance-break hints; sentence-end punctuation handles
                                                    // line breaks in the store.)
                                                    if t.text.starts_with('<') && t.text.ends_with('>') {
                                                        continue;
                                                    }
                                                    if t.is_final {
                                                        let _ = event_tx
                                                            .send(TranslationEvent::Final {
                                                                text: t.text,
                                                                language: t.language,
                                                                status: t.translation_status,
                                                                speaker: t.speaker,
                                                                ts_ms: SonioxBYOK::now_ms(),
                                                            })
                                                            .await;
                                                    } else if matches!(t.translation_status, TranslationStatus::Translation) {
                                                        trans_text.push_str(&t.text);
                                                        trans_meta = Some((t.language, t.speaker));
                                                    } else {
                                                        orig_text.push_str(&t.text);
                                                        orig_meta = Some((t.language, t.speaker));
                                                    }
                                                }
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
                                                let _ = event_tx
                                                    .send(stopped_with_err(format!("Soniox error {code}: {message}")))
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

                // Reconnect with fresh attempt counter (1-indexed for the new round).
                attempt = 1;
                let wait = next_backoff_ms(attempt);
                let _ = event_tx
                    .send(TranslationEvent::Reconnecting {
                        attempt,
                        retry_in_ms: wait,
                    })
                    .await;
                tokio::time::sleep(Duration::from_millis(wait)).await;
            }
        });

        self.task = Some(task);
        Ok(())
    }

    async fn send_audio(&mut self, pcm: &[u8]) -> Result<()> {
        let Some(tx) = self.audio_tx.as_ref() else {
            return Err(Error::Soniox("provider not open".into()));
        };
        tx.send(Outbound::Binary(pcm.to_vec()))
            .await
            .map_err(|e| Error::Soniox(format!("audio channel closed: {e}")))
    }

    async fn next_event(&mut self) -> Option<TranslationEvent> {
        let rx = self.event_rx.as_mut()?;
        rx.recv().await
    }

    async fn close(&mut self) -> Result<()> {
        // Dropping the sender closes the channel. The background task detects
        // `audio_rx.recv() -> None` and treats it as client EOS: it forwards an
        // EOS sentinel to the server and exits cleanly without needing abort().
        self.audio_tx = None;

        let result = if let Some(task) = self.task.take() {
            match tokio::time::timeout(Duration::from_secs(3), task).await {
                Ok(Ok(())) => Ok(()),
                Ok(Err(join_err)) => Err(Error::Soniox(format!("join: {join_err}"))),
                Err(_) => Err(Error::Soniox("close timed out".into())),
            }
        } else {
            Ok(())
        };

        // Always clear the event channel so next_event() returns None immediately.
        self.event_rx = None;
        result
    }
}
