#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use parking_lot::Mutex;
use tauri::Manager;

mod commands;
mod events;
mod state;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::try_init().ok();

    let app_state = state::init().await.expect("voxtide-core init");
    let app_state = Mutex::new(Some(app_state));

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_positioner::init())
        .setup(move |app| {
            // Hand state off to Tauri.
            let state = app_state.lock().take().expect("AppState already taken");
            app.manage(state);
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
            commands::lifecycle::start_session,
            commands::lifecycle::stop_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
