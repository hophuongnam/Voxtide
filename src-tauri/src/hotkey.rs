use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

/// Register the global hotkey (Cmd/Ctrl+Shift+V) that toggles the active session.
///
/// When triggered, emits `voxtide://hotkey/toggle` to all windows. The main window
/// listens for this event and calls `onStart` or `onStop` depending on recording state.
pub fn register(app: &AppHandle) -> tauri::Result<()> {
    let app_clone = app.clone();
    app.global_shortcut()
        .on_shortcut("CommandOrControl+Shift+V", move |_app, _shortcut, _ev| {
            let _ = app_clone.emit("voxtide://hotkey/toggle", ());
        })
        .map_err(|e| tauri::Error::PluginInitialization("global-shortcut".into(), e.to_string()))?;
    Ok(())
}
