use tauri::{AppHandle, Manager};

#[tauri::command]
pub fn show_overlay(app: AppHandle) -> Result<(), String> {
    let w = app
        .get_webview_window("overlay")
        .ok_or("overlay window missing")?;
    // Reset to interactive every show: a prior `set_overlay_click_through(true)` would
    // otherwise persist across hide+show and swallow the mouseenter that reveals the
    // drag handle, leaving the overlay stuck where it appeared.
    w.set_ignore_cursor_events(false)
        .map_err(|e| e.to_string())?;
    w.show().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn hide_overlay(app: AppHandle) -> Result<(), String> {
    let w = app
        .get_webview_window("overlay")
        .ok_or("overlay window missing")?;
    w.hide().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_overlay_click_through(app: AppHandle, click_through: bool) -> Result<(), String> {
    let w = app
        .get_webview_window("overlay")
        .ok_or("overlay window missing")?;
    w.set_ignore_cursor_events(click_through)
        .map_err(|e| e.to_string())
}
