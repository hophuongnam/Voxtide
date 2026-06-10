use voxtide_core::config::{AppConfig, ConfigStore, FontSize, Theme};
use voxtide_core::translation::Mode;

#[test]
fn default_config_has_expected_shape() {
    let cfg = AppConfig::default();
    assert_eq!(cfg.language_a, "en");
    assert_eq!(cfg.language_b, "vi");
    // A valid plugin accelerator meaning ⌘⇧V on macOS / Ctrl+Shift+V
    // elsewhere — what registration has always actually bound.
    assert_eq!(cfg.hotkey, "CommandOrControl+Shift+V");
    assert!(matches!(cfg.theme, Theme::System));
    assert!(cfg.default_meeting_source.is_none());
    assert!(cfg.default_mic.is_none());
    assert!(matches!(cfg.mode, Mode::Meeting));
}

#[test]
fn stale_write_only_hotkey_default_migrates_on_load() {
    // Every config.json written before the hotkey field was honored carries
    // the old default "Ctrl+Shift+V" (the field was write-only; registration
    // hardcoded CommandOrControl+Shift+V). Loading must rewrite exactly that
    // string to the accelerator the app actually bound, or existing macOS
    // installs would silently switch from ⌘⇧V to ⌃⇧V.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let cfg = AppConfig {
        hotkey: "Ctrl+Shift+V".into(),
        ..AppConfig::default()
    };
    std::fs::write(&path, serde_json::to_vec_pretty(&cfg).unwrap()).unwrap();
    let loaded = ConfigStore::at(&path).load().unwrap();
    assert_eq!(loaded.hotkey, "CommandOrControl+Shift+V");

    // A genuinely custom binding is left alone.
    let cfg = AppConfig {
        hotkey: "Alt+F5".into(),
        ..AppConfig::default()
    };
    std::fs::write(&path, serde_json::to_vec_pretty(&cfg).unwrap()).unwrap();
    let loaded = ConfigStore::at(&path).load().unwrap();
    assert_eq!(loaded.hotkey, "Alt+F5");
}

#[test]
fn pre_mode_field_config_json_still_loads() {
    // Simulates a config.json written by v0.1.1 (before the `mode` field existed).
    // Should round-trip successfully and default to Meeting.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    std::fs::write(
        &path,
        r#"{
  "language_a": "en",
  "language_b": "vi",
  "mine": "b",
  "hotkey": "Ctrl+Shift+V",
  "theme": "system",
  "default_meeting_source": null,
  "default_mic": null
}"#,
    )
    .unwrap();
    let loaded = ConfigStore::at(&path).load().unwrap();
    assert!(matches!(loaded.mode, Mode::Meeting));
}

#[test]
fn mode_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let store = ConfigStore::at(dir.path().join("config.json"));
    let cfg = AppConfig {
        mode: Mode::Conversation,
        ..AppConfig::default()
    };
    store.save(&cfg).unwrap();
    let loaded = store.load().unwrap();
    assert!(matches!(loaded.mode, Mode::Conversation));
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

#[test]
fn default_config_has_reading_defaults() {
    let cfg = AppConfig::default();
    assert!(matches!(cfg.font_size, FontSize::M));
    assert!(!cfg.show_pinyin);
}

#[test]
fn pre_reading_fields_config_json_still_loads() {
    // Simulates a config.json written before font_size/show_pinyin existed.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    std::fs::write(
        &path,
        r#"{
  "language_a": "en",
  "language_b": "vi",
  "mine": "b",
  "hotkey": "Ctrl+Shift+V",
  "theme": "system",
  "default_meeting_source": null,
  "default_mic": null,
  "mode": "meeting"
}"#,
    )
    .unwrap();
    let loaded = ConfigStore::at(&path).load().unwrap();
    assert!(matches!(loaded.font_size, FontSize::M));
    assert!(!loaded.show_pinyin);
}

#[test]
fn reading_fields_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let store = ConfigStore::at(dir.path().join("config.json"));
    let cfg = AppConfig {
        font_size: FontSize::Xl,
        show_pinyin: true,
        ..AppConfig::default()
    };
    store.save(&cfg).unwrap();
    let loaded = store.load().unwrap();
    assert!(matches!(loaded.font_size, FontSize::Xl));
    assert!(loaded.show_pinyin);
}
