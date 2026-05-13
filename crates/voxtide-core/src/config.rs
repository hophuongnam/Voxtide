use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::translation::WhichLang;
use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Theme {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    pub language_a: String,
    pub language_b: String,
    pub mine: WhichLang,
    pub hotkey: String,
    pub theme: Theme,
    pub default_meeting_source: Option<String>,
    pub default_mic: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language_a: "en".into(),
            language_b: "vi".into(),
            mine: WhichLang::B,
            hotkey: "Ctrl+Shift+V".into(),
            theme: Theme::System,
            default_meeting_source: None,
            default_mic: None,
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
            Ok(b) => serde_json::from_slice(&b).map_err(Error::from),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(AppConfig::default()),
            Err(e) => Err(Error::from(e)),
        }
    }
}
