#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use parking_lot::Mutex;
use tauri::Manager;

mod commands;
mod events;
mod hotkey;
mod state;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::try_init().ok();

    let app_state = state::init().await.expect("voxtide-core init");
    let app_state = Mutex::new(Some(app_state));

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_positioner::init())
        .on_window_event(|window, event| {
            // macOS pattern: red-traffic-light closes the window but keeps the app
            // running in the dock; dock-click re-shows it (handled in the run loop).
            // Cmd+Q / "Quit Voxtide" fire ExitRequested, not CloseRequested, so quit
            // still works as expected.
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .setup(move |app| {
            let state = app_state.lock().take().expect("AppState already taken");
            // Subscribe BEFORE handing state to Tauri so we hold a reference to the controller.
            // This single persistent forwarder replaces the per-call spawns that were previously
            // in `lifecycle::start_session`, which leaked one task per start/stop cycle.
            let mut rx = state.controller.subscribe();
            let app_handle = app.handle().clone();
            tokio::spawn(async move {
                loop {
                    match rx.recv().await {
                        Ok(ev) => crate::events::forward(&app_handle, ev),
                        // Lagged: some events were dropped because we fell behind the sender.
                        // Treat as a refresh signal — continue rather than breaking.
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                        // Channel closed (controller dropped). Forwarder task can exit.
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
            });
            app.manage(state);
            // Register the global hotkey on startup.
            hotkey::register(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::keychain::has_api_key,
            commands::keychain::set_api_key,
            commands::keychain::clear_api_key,
            commands::config::get_config,
            commands::config::set_config,
            commands::devices::list_mics,
            commands::devices::list_loopback_sources,
            commands::sessions::list_sessions,
            commands::sessions::get_session,
            commands::sessions::search_transcripts,
            commands::sessions::delete_session,
            commands::lifecycle::start_session,
            commands::lifecycle::stop_session,
            commands::overlay::show_overlay,
            commands::overlay::hide_overlay,
            commands::overlay::set_overlay_click_through,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Reopen {
            has_visible_windows,
            ..
        } = event
        {
            if !has_visible_windows {
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (app_handle, event);
        }
    });
}
