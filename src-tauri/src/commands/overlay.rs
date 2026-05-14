use tauri::{AppHandle, Manager};

#[tauri::command]
pub fn show_overlay(app: AppHandle) -> Result<(), String> {
    let w = app
        .get_webview_window("overlay")
        .ok_or("overlay window missing")?;
    w.show().map_err(|e| e.to_string())?;
    w.set_ignore_cursor_events(true)
        .map_err(|e| e.to_string())?;
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
