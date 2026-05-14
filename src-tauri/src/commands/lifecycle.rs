use serde::Deserialize;
use tauri::{AppHandle, State};

use voxtide_core::audio::{mic::MicSource, AudioSource};
use voxtide_core::session::StartArgs;
use voxtide_core::translation::soniox::SonioxBYOK;
use voxtide_core::translation::{Mode, SessionConfig, WhichLang};

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct StartReq {
    pub mode: Mode,
    pub language_a: String,
    pub language_b: String,
    pub mine: WhichLang,
    pub device_id: String,
    pub api_key_account: String,
}

#[tauri::command]
pub async fn start_session(
    app: AppHandle,
    state: State<'_, AppState>,
    req: StartReq,
) -> Result<i64, String> {
    let api_key = state
        .keychain
        .get(&req.api_key_account)
        .map_err(|e| e.to_string())?;
    let source: Box<dyn AudioSource> = match req.mode {
        Mode::Conversation => Box::new(MicSource::by_id(&req.device_id)),
        Mode::Meeting => loopback_source(&req.device_id)?,
    };
    let provider = Box::new(SonioxBYOK::new());

    // Subscribe before start so we don't miss the first event.
    let mut rx = state.controller.subscribe();
    let app_for_fwd = app.clone();
    tokio::spawn(async move {
        while let Ok(ev) = rx.recv().await {
            crate::events::forward(&app_for_fwd, ev);
        }
    });

    let cfg = SessionConfig {
        api_key,
        mode: req.mode,
        language_a: req.language_a,
        language_b: req.language_b,
        mine: req.mine,
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
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_session(state: State<'_, AppState>) -> Result<(), String> {
    state.controller.stop().await.map_err(|e| e.to_string())
}

#[cfg(target_os = "macos")]
fn loopback_source(device_id: &str) -> Result<Box<dyn AudioSource>, String> {
    use voxtide_core::audio::macos_loopback::{list_loopback_sources, MacLoopbackSource};
    let sources = list_loopback_sources().map_err(|e| e.to_string())?;
    let target = sources
        .into_iter()
        .find(|s| s.id == device_id)
        .ok_or_else(|| format!("loopback source not found: {device_id}"))?;
    Ok(Box::new(MacLoopbackSource::new(target)))
}

#[cfg(target_os = "windows")]
fn loopback_source(device_id: &str) -> Result<Box<dyn AudioSource>, String> {
    use voxtide_core::audio::windows_loopback::WinLoopbackSource;
    Ok(Box::new(WinLoopbackSource::by_id(device_id)))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn loopback_source(_id: &str) -> Result<Box<dyn AudioSource>, String> {
    Err("loopback unsupported on this platform".into())
}
