use serde::Deserialize;
use tauri::State;

use voxtide_core::audio::{mic::MicSource, AudioSource};
use voxtide_core::session::StartArgs;
use voxtide_core::translation::soniox::SonioxBYOK;
use voxtide_core::translation::{Mode, SessionConfig};
use voxtide_core::Error as CoreError;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct StartReq {
    pub mode: Mode,
    pub language_a: String,
    pub language_b: String,
    pub device_id: String,
    pub api_key_account: String,
}

/// Structured failure returned by [`start_session`]. Tauri serializes the `Err`
/// value as the promise-rejection payload, so the frontend can route precisely
/// (permission banner vs. plain error strip) instead of sniffing substrings.
///
/// `kind` is the routing discriminator; `message` is the raw human-readable
/// detail (also shown verbatim for `other` / `device-missing`).
#[derive(Debug, serde::Serialize)]
pub struct StartError {
    pub kind: &'static str,
    pub message: String,
}

impl StartError {
    /// Classify a typed core error, given the source the user requested.
    ///
    /// - any `Audio` error mentioning "not found" → `device-missing` (the
    ///   selected mic/loopback device vanished, e.g. unplugged) — distinct from
    ///   a permission denial.
    /// - other `Audio` errors → a permission problem, scoped to the requested
    ///   source: `mic-permission` for the microphone, `capture-permission` for
    ///   system-audio loopback.
    /// - anything else (Soniox auth/handshake, persistence, session) → `other`.
    fn classify(err: &CoreError, mode: Mode) -> Self {
        let message = err.to_string();
        let kind = match err {
            CoreError::Audio(detail) if detail.contains("not found") => "device-missing",
            CoreError::Audio(_) => match mode {
                Mode::Conversation => "mic-permission",
                Mode::Meeting => "capture-permission",
            },
            _ => "other",
        };
        StartError { kind, message }
    }
}

#[tauri::command]
pub async fn start_session(state: State<'_, AppState>, req: StartReq) -> Result<i64, StartError> {
    let mode = req.mode;
    let api_key = state
        .keychain
        .get(&req.api_key_account)
        .map_err(|e| StartError::classify(&e, mode))?;
    // For Conversation mode, fall back to the default microphone when no specific device is
    // requested. An empty `device_id` from the frontend means "use the system default".
    let source: Box<dyn AudioSource> = match req.mode {
        Mode::Conversation => {
            if req.device_id.is_empty() {
                Box::new(MicSource::default_device().map_err(|e| StartError::classify(&e, mode))?)
            } else {
                Box::new(MicSource::by_id(&req.device_id))
            }
        }
        Mode::Meeting => {
            loopback_source(&req.device_id).map_err(|e| StartError::classify(&e, mode))?
        }
    };
    let provider = Box::new(SonioxBYOK::new());
    let cfg = SessionConfig {
        api_key,
        mode: req.mode,
        language_a: req.language_a,
        language_b: req.language_b,
    };
    state
        .controller
        .start(StartArgs {
            cfg,
            source,
            provider,
            device_label: Some(req.device_id),
        })
        .await
        .map_err(|e| StartError::classify(&e, mode))
}

#[tauri::command]
pub async fn stop_session(state: State<'_, AppState>) -> Result<(), String> {
    state.controller.stop().await.map_err(|e| e.to_string())
}

#[cfg(target_os = "macos")]
fn loopback_source(device_id: &str) -> Result<Box<dyn AudioSource>, CoreError> {
    use voxtide_core::audio::macos_loopback::{list_loopback_sources, MacLoopbackSource};
    let sources = list_loopback_sources()?;
    let target = sources
        .into_iter()
        .find(|s| s.id == device_id)
        .ok_or_else(|| CoreError::Audio(format!("loopback source not found: {device_id}")))?;
    Ok(Box::new(MacLoopbackSource::new(target)))
}

#[cfg(target_os = "windows")]
fn loopback_source(device_id: &str) -> Result<Box<dyn AudioSource>, CoreError> {
    use voxtide_core::audio::windows_loopback::WinLoopbackSource;
    Ok(Box::new(WinLoopbackSource::by_id(device_id)))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn loopback_source(_id: &str) -> Result<Box<dyn AudioSource>, CoreError> {
    Err(CoreError::Audio(
        "loopback unsupported on this platform".into(),
    ))
}
