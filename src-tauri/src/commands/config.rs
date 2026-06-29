use tauri::{Emitter, State};

use voxtide_core::config::AppConfig;

use crate::state::AppState;

/// Structured command failure: `kind` lets the frontend branch without string
/// sniffing (same pattern as lifecycle's `StartError`). Tauri serializes the
/// `Err` payload as the promise rejection value.
#[derive(serde::Serialize)]
pub struct ConfigError {
    pub kind: &'static str,
    pub message: String,
}

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    state.config.load().map_err(|e| e.to_string())
}

/// Static facts the status bar displays. Sourced from the core crate's real
/// constants so the UI can never drift from what the pipeline actually does
/// (the model string and audio format used to be hardcoded frontend literals).
#[derive(serde::Serialize)]
pub struct AppInfo {
    pub model: &'static str,
    pub sample_rate_hz: u32,
    pub channels: u16,
}

#[tauri::command]
pub fn app_info() -> AppInfo {
    AppInfo {
        model: voxtide_core::translation::soniox::MODEL,
        sample_rate_hz: voxtide_core::audio::SAMPLE_RATE_HZ,
        channels: voxtide_core::audio::CHANNELS,
    }
}

#[tauri::command]
pub fn set_config(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    cfg: AppConfig,
) -> Result<(), ConfigError> {
    #[cfg(desktop)]
    let old_hotkey = state.config.load().ok().map(|c| c.hotkey);
    state.config.save(&cfg).map_err(|e| ConfigError {
        kind: "io",
        message: e.to_string(),
    })?;
    // Broadcast the persisted config so other windows (overlay) re-derive
    // their labels/theme/hotkey hint live. Emitted right after save — the
    // file holds this config even if the hotkey re-register below fails.
    let _ = app.emit("voxtide://config", &cfg);
    // Apply a hotkey change live (no restart). The config STAYS SAVED even if
    // registration fails — the user corrects the field with the error shown —
    // but the previous working binding is restored best-effort so a typo
    // doesn't strand them with no hotkey at all.
    #[cfg(desktop)]
    {
        if old_hotkey.as_deref() != Some(cfg.hotkey.as_str()) {
            if let Err(e) = crate::hotkey::reregister(&app, &cfg.hotkey) {
                if let Some(old) = old_hotkey {
                    let _ = crate::hotkey::reregister(&app, &old);
                }
                return Err(ConfigError {
                    kind: "invalid-hotkey",
                    message: format!("could not register '{}': {e}", cfg.hotkey),
                });
            }
        }
    }
    Ok(())
}
