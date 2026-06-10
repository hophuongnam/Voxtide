use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::translation::Mode;
use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Theme {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum FontSize {
    Xs,
    S,
    #[default]
    M,
    L,
    Xl,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    pub language_a: String,
    pub language_b: String,
    pub hotkey: String,
    pub theme: Theme,
    pub default_meeting_source: Option<String>,
    pub default_mic: Option<String>,
    // Persisted last-used capture mode. `#[serde(default)]` so pre-0.1.2 config.json
    // files that pre-date this field still load (and start in Meeting).
    #[serde(default)]
    pub mode: Mode,
    #[serde(default)]
    pub font_size: FontSize,
    #[serde(default)]
    pub show_pinyin: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language_a: "en".into(),
            language_b: "vi".into(),
            // A valid global-shortcut-plugin accelerator: ⌘⇧V on macOS,
            // Ctrl+Shift+V elsewhere — the binding registration always used.
            hotkey: "CommandOrControl+Shift+V".into(),
            theme: Theme::System,
            default_meeting_source: None,
            default_mic: None,
            mode: Mode::Meeting,
            font_size: FontSize::M,
            show_pinyin: false,
        }
    }
}

pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    pub fn at<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn save(&self, cfg: &AppConfig) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec_pretty(cfg)?;
        std::fs::write(&self.path, bytes)?;
        Ok(())
    }

    pub fn load(&self) -> Result<AppConfig> {
        match std::fs::read(&self.path) {
            Ok(b) => {
                let mut cfg: AppConfig = serde_json::from_slice(&b).map_err(Error::from)?;
                // Migration shim: before the hotkey field was honored it was
                // write-only — every config.json carries the old default
                // "Ctrl+Shift+V" while registration hardcoded
                // CommandOrControl+Shift+V (⌘⇧V on macOS). Registering the
                // stored string literally would silently switch existing macOS
                // installs to ⌃⇧V, so exactly the old default is rewritten to
                // the accelerator the app actually bound. Nobody deliberately
                // chose "Ctrl+Shift+V" (the field did nothing), and on Windows
                // the two are identical anyway.
                if cfg.hotkey == "Ctrl+Shift+V" {
                    cfg.hotkey = "CommandOrControl+Shift+V".into();
                }
                Ok(cfg)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(AppConfig::default()),
            Err(e) => Err(Error::from(e)),
        }
    }
}
