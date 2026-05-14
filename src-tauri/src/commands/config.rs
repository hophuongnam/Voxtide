use tauri::State;

use voxtide_core::config::AppConfig;

use crate::state::AppState;

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    state.config.load().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_config(state: State<'_, AppState>, cfg: AppConfig) -> Result<(), String> {
    state.config.save(&cfg).map_err(|e| e.to_string())
}
