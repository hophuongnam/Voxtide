use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranslationStatus {
    Original,
    Translation,
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Token {
    pub text: String,
    #[serde(default)]
    pub is_final: bool,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default = "default_status")]
    pub translation_status: TranslationStatus,
    #[serde(default)]
    pub source_language: Option<String>,
    #[serde(default)]
    pub speaker: Option<String>,
}

fn default_status() -> TranslationStatus {
    TranslationStatus::None
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokensFrame {
    #[serde(default)]
    pub tokens: Vec<Token>,
    #[serde(default)]
    pub final_audio_proc_ms: Option<u64>,
    #[serde(default)]
    pub total_audio_proc_ms: Option<u64>,
    #[serde(default)]
    pub finished: bool,
}

#[derive(Debug, Clone)]
pub enum ServerMessage {
    Tokens(TokensFrame),
    Finished,
    Error { code: String, message: String },
}

pub fn parse_message(s: &str) -> crate::Result<ServerMessage> {
    let v: serde_json::Value = serde_json::from_str(s)?;
    if let Some(err) = v.get("error") {
        let code = err
            .get("code")
            .and_then(|x| x.as_str())
            .unwrap_or("unknown")
            .to_string();
        let message = err
            .get("message")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        return Ok(ServerMessage::Error { code, message });
    }
    // Plain `{ "finished": true }` (no tokens) is the end-of-stream marker.
    if v.get("tokens").is_none() && v.get("finished").and_then(|f| f.as_bool()) == Some(true) {
        return Ok(ServerMessage::Finished);
    }
    let frame: TokensFrame = serde_json::from_value(v)?;
    Ok(ServerMessage::Tokens(frame))
}
