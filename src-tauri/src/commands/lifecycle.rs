use serde::Deserialize;
use tauri::State;

use voxtide_core::audio::{mic::MicSource, mix::MixSource, AudioSource};
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
    /// System Audio mode only: also capture the local microphone, blended into
    /// the system-audio stream (the session then runs two-way).
    #[serde(default)]
    pub capture_mic: bool,
    /// Which mic to blend when `capture_mic` is set; empty = system default.
    #[serde(default)]
    pub mic_device_id: String,
    /// Optional free-text context (names, jargon, domain) → Soniox, to bias
    /// recognition and translation. Empty = omitted from the wire config.
    #[serde(default)]
    pub context: String,
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
    /// - any `Audio` error mentioning "not found" or "no default input device"
    ///   → `device-missing` (selected mic/loopback vanished, or machine has no
    ///   mic at all) — distinct from a permission denial.
    /// - other `Audio` errors → a permission problem, scoped to the requested
    ///   source: `mic-permission` for the microphone, `capture-permission` for
    ///   system-audio loopback.
    /// - anything else (Soniox auth/handshake, persistence, session) → `other`.
    ///
    /// IMPORTANT: any new `Error::Audio` message that means "device absent" MUST
    /// contain either "not found" or "no default input device" — these substrings
    /// are load-bearing: the frontend uses `device-missing` to show the plain
    /// error strip (not the permission banner). Likewise, a mic-leg failure of a
    /// System Audio blend is marked with a "microphone:" prefix by
    /// `MixSource::start`, so a mic permission denial routes to the microphone
    /// banner rather than the system-audio one.
    fn classify(err: &CoreError, mode: Mode) -> Self {
        let message = err.to_string();
        let kind = match err {
            CoreError::Audio(detail)
                if detail.contains("not found") || detail.contains("no default input device") =>
            {
                "device-missing"
            }
            // Mic leg of a System Audio blend failed (marked by MixSource).
            CoreError::Audio(detail) if detail.contains("microphone:") => "mic-permission",
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
            #[cfg(target_os = "android")]
            {
                let _ = &req.device_id; // device selection N/A on Android (single WebView mic)
                Box::new(voxtide_core::audio::webview_mic::WebViewMicSource::new(
                    state.mic_feed.clone(),
                ))
            }
            #[cfg(not(target_os = "android"))]
            {
                if req.device_id.is_empty() {
                    Box::new(
                        MicSource::default_device().map_err(|e| StartError::classify(&e, mode))?,
                    )
                } else {
                    Box::new(MicSource::by_id(&req.device_id))
                }
            }
        }
        Mode::Meeting => {
            let loopback = voxtide_core::audio::loopback::by_id(&req.device_id)
                .map_err(|e| StartError::classify(&e, mode))?;
            if req.capture_mic {
                // Blend the local mic in (default device when none specified).
                // The mic is the secondary/overlay; the loopback is the clock.
                let mic: Box<dyn AudioSource> = if req.mic_device_id.is_empty() {
                    Box::new(
                        MicSource::default_device().map_err(|e| StartError::classify(&e, mode))?,
                    )
                } else {
                    Box::new(MicSource::by_id(&req.mic_device_id))
                };
                Box::new(MixSource::new(loopback, mic))
            } else {
                loopback
            }
        }
    };
    let provider = Box::new(SonioxBYOK::new().with_context(req.context));
    let cfg = SessionConfig {
        api_key,
        mode: req.mode,
        language_a: req.language_a,
        language_b: req.language_b,
        capture_mic: req.capture_mic,
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

/// Mid-session context switch: routes to the running session's provider via
/// `SessionController::update_context`, which is itself a best-effort no-op
/// when no session is active — so this command is infallible too.
#[tauri::command]
pub async fn update_context(text: String, state: State<'_, AppState>) -> Result<(), String> {
    state.controller.update_context(text).await;
    Ok(())
}
