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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
