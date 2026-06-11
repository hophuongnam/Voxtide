use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use voxtide_core::config::ConfigStore;
use voxtide_core::persistence::Store;
use voxtide_core::session::SessionController;
use voxtide_core::Keychain;

pub struct AppState {
    pub controller: Arc<SessionController>,
    pub keychain: Keychain,
    pub config: ConfigStore,
    /// Tracks the overlay window's visibility (set by the show/hide
    /// commands) so the event forwarder can skip emitting every token
    /// event into a hidden webview.
    pub overlay_visible: Arc<AtomicBool>,
}

pub fn data_dir() -> PathBuf {
    dirs::data_dir()
        .map(|d| d.join("Voxtide"))
        .unwrap_or_else(|| PathBuf::from("./voxtide-data"))
}

pub async fn init() -> voxtide_core::Result<AppState> {
    let dir = data_dir();
    std::fs::create_dir_all(&dir)?;
    let store = Store::open(&dir.join("voxtide.db")).await?;
    Ok(AppState {
        controller: Arc::new(SessionController::new(store)),
        keychain: Keychain::new(dir.join("secrets.json")),
        config: ConfigStore::at(dir.join("config.json")),
        overlay_visible: Arc::new(AtomicBool::new(false)),
    })
}
