use voxtide_core::config::{AppConfig, ConfigStore, Theme};
use voxtide_core::translation::WhichLang;

#[test]
fn default_config_has_expected_shape() {
    let cfg = AppConfig::default();
    assert_eq!(cfg.language_a, "en");
    assert_eq!(cfg.language_b, "vi");
    assert!(matches!(cfg.mine, WhichLang::B));
    assert_eq!(cfg.hotkey, "Ctrl+Shift+V");
    assert!(matches!(cfg.theme, Theme::System));
    assert!(cfg.default_meeting_source.is_none());
    assert!(cfg.default_mic.is_none());
}

#[test]
fn save_and_reload_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let store = ConfigStore::at(dir.path().join("config.json"));
    let cfg = AppConfig {
        language_a: "ja".into(),
        theme: Theme::Dark,
        default_mic: Some("MacBook Pro Mic".into()),
        ..AppConfig::default()
    };
    store.save(&cfg).unwrap();

    let loaded = store.load().unwrap();
    assert_eq!(loaded.language_a, "ja");
    assert!(matches!(loaded.theme, Theme::Dark));
    assert_eq!(loaded.default_mic.as_deref(), Some("MacBook Pro Mic"));
}

#[test]
fn loading_missing_file_returns_default() {
    let dir = tempfile::tempdir().unwrap();
    let store = ConfigStore::at(dir.path().join("missing.json"));
    let cfg = store.load().unwrap();
    assert_eq!(cfg, AppConfig::default());
}
