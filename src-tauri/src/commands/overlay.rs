use std::sync::atomic::Ordering;

use tauri::{AppHandle, Emitter, Manager, State};

use crate::state::AppState;

/// Record + broadcast the overlay's new visibility: the AtomicBool gates the
/// event forwarder (no token events into a hidden webview), and the event
/// keeps every window's UI state (e.g. the toolbar toggle) tracking the REAL
/// window state instead of a local guess.
fn set_visibility(app: &AppHandle, state: &State<'_, AppState>, visible: bool) {
    state.overlay_visible.store(visible, Ordering::Relaxed);
    let _ = app.emit(
        "voxtide://overlay",
        serde_json::json!({ "visible": visible }),
    );
}

#[tauri::command]
pub fn show_overlay(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let w = app
        .get_webview_window("overlay")
        .ok_or("overlay window missing")?;
    // Reset to interactive every show: a prior `set_overlay_click_through(true)` would
    // otherwise persist across hide+show and swallow the mouseenter that reveals the
    // drag handle, leaving the overlay stuck where it appeared.
    w.set_ignore_cursor_events(false)
        .map_err(|e| e.to_string())?;
    w.show().map_err(|e| e.to_string())?;
    set_visibility(&app, &state, true);
    Ok(())
}

#[tauri::command]
pub fn hide_overlay(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let w = app
        .get_webview_window("overlay")
        .ok_or("overlay window missing")?;
    w.hide().map_err(|e| e.to_string())?;
    set_visibility(&app, &state, false);
    Ok(())
}

#[tauri::command]
pub fn set_overlay_click_through(app: AppHandle, click_through: bool) -> Result<(), String> {
    let w = app
        .get_webview_window("overlay")
        .ok_or("overlay window missing")?;
    w.set_ignore_cursor_events(click_through)
        .map_err(|e| e.to_string())
}
