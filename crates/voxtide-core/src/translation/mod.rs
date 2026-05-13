//! Translation provider abstraction.

pub mod tokens;
pub mod soniox;
pub mod mock;

use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Meeting,
    Conversation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub api_key: String,
    pub mode: Mode,
    pub language_a: String,
    pub language_b: String,
    pub mine: WhichLang,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WhichLang {
    A,
    B,
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
