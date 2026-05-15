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
    Stopped,
}

#[async_trait::async_trait]
pub trait TranslationProvider: Send {
    async fn open(&mut self, cfg: SessionConfig) -> Result<()>;
    async fn send_audio(&mut self, pcm: &[u8]) -> Result<()>;
    async fn next_event(&mut self) -> Option<TranslationEvent>;
    async fn close(&mut self) -> Result<()>;
}
