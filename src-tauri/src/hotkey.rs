use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// Register `accel` (a plugin accelerator like `CommandOrControl+Shift+V`) as
/// the global hotkey that toggles the active session.
///
/// Emits `voxtide://hotkey/toggle` to all windows on key PRESS only — the
/// plugin dispatches both `Pressed` and `Released`, and reacting to both
/// double-fired the toggle (one keypress started AND immediately stopped a
/// session). The main window listens for this event and calls `onStart` or
/// `onStop` depending on recording state.
///
/// Errors (unparsable accelerator, OS-level conflict with another app's
/// shortcut) are returned to the caller; they are expected runtime conditions,
/// not panics.
pub fn register(app: &AppHandle, accel: &str) -> tauri::Result<()> {
    let app_clone = app.clone();
    app.global_shortcut()
        .on_shortcut(accel, move |_app, _shortcut, ev| {
            if ev.state == ShortcutState::Pressed {
                let _ = app_clone.emit("voxtide://hotkey/toggle", ());
            }
        })
        .map_err(|e| tauri::Error::PluginInitialization("global-shortcut".into(), e.to_string()))?;
    Ok(())
}

/// Swap the global hotkey to a new accelerator (Settings change, applied live
/// — no restart). Unregisters everything first; the toggle is the only
/// shortcut this app ever registers.
pub fn reregister(app: &AppHandle, accel: &str) -> tauri::Result<()> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| tauri::Error::PluginInitialization("global-shortcut".into(), e.to_string()))?;
    register(app, accel)
}
