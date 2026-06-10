//! Translation provider abstraction.

pub mod mock;
pub mod soniox;
pub mod tokens;

use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    #[default]
    Meeting,
    Conversation,
}

/// `language_a` is the source (spoken) language; `language_b` is the
/// translation target. Meeting mode translates a → b one-way;
/// Conversation mode translates both directions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub api_key: String,
    pub mode: Mode,
    pub language_a: String,
    pub language_b: String,
}

#[derive(Debug, Clone)]
pub enum TranslationEvent {
    Final {
        text: String,
        language: Option<String>,
        status: tokens::TranslationStatus,
        speaker: Option<String>,
        ts_ms: i64,
    },
    Live {
        text: String,
        language: Option<String>,
        status: tokens::TranslationStatus,
        speaker: Option<String>,
    },
    /// Soniox endpoint detection fired — a speech pause delimiting one
    /// utterance from the next. Carries no text; the store uses it to start
    /// a new transcript row in both columns.
    UtteranceBreak,
    Connected,
    Reconnecting {
        attempt: u32,
        retry_in_ms: u64,
    },
    /// A terminal provider failure with a human-readable message (e.g. a Soniox
    /// server error like a bad API key or exhausted quota). The session worker
    /// rebroadcasts this as [`crate::session::CoreEvent::Error`]; a `Stopped`
    /// follows to end the session normally.
    Error(String),
    Stopped,
}

#[async_trait::async_trait]
pub trait TranslationProvider: Send {
    async fn open(&mut self, cfg: SessionConfig) -> Result<()>;
    /// Send one PCM chunk to the provider. Takes ownership of the buffer so the
    /// implementation can move it straight into its outbound channel / wire
    /// frame without re-copying (the call site already builds an owned `Vec`).
    async fn send_audio(&mut self, pcm: Vec<u8>) -> Result<()>;
    async fn next_event(&mut self) -> Option<TranslationEvent>;
    /// Initiate end-of-stream WITHOUT tearing the provider down: signal the
    /// remote that no more audio is coming, but keep [`next_event`] live so the
    /// flushed trailing finals (the last words spoken before stop) still drain
    /// to the caller. The session worker calls this on explicit stop, drains the
    /// remaining events, and only then calls [`close`].
    ///
    /// Default impl is a no-op for providers that flush nothing. Must be
    /// idempotent: calling it more than once (or after `close`) is harmless.
    async fn eos(&mut self) {}
    async fn close(&mut self) -> Result<()>;
}
