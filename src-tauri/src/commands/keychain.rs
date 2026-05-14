use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn has_api_key(state: State<'_, AppState>, account: String) -> Result<bool, String> {
    match state.keychain.get(&account) {
        Ok(s) => Ok(!s.is_empty()),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
pub fn set_api_key(
    state: State<'_, AppState>,
    account: String,
    secret: String,
) -> Result<(), String> {
    state
        .keychain
        .set(&account, &secret)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_api_key(state: State<'_, AppState>, account: String) -> Result<(), String> {
    state.keychain.delete(&account).map_err(|e| e.to_string())
}
