use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Soniox: {0}")]
    Soniox(String),

    #[error("audio: {0}")]
    Audio(String),

    #[error("persistence: {0}")]
    Persistence(#[from] sqlx::Error),

    #[error("migration: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("keychain: {0}")]
    Keychain(#[from] keyring::Error),

    #[error("config: {0}")]
    Config(String),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("json: {0}")]
    Json(#[from] serde_json::Error),

    #[error("websocket: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("invalid url: {0}")]
    Url(#[from] url::ParseError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
