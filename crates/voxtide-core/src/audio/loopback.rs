//! Platform-free system-audio loopback facade.
//!
//! The `#[cfg]` dispatch lives HERE, so the Tauri shell's commands and
//! lifecycle code stay single-implementation (they previously each carried
//! three per-platform variants of the same list/construct logic).

use crate::audio::AudioSource;
use crate::Result;

/// One selectable loopback capture endpoint, shape-stable across platforms.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoopbackDevice {
    pub id: String,
    pub label: String,
    pub default: bool,
}

/// List the system-audio endpoints available for loopback capture.
pub fn list() -> Result<Vec<LoopbackDevice>> {
    #[cfg(target_os = "macos")]
    {
        // ScreenCaptureKit captures the whole render path, so macOS exposes a
        // single synthetic entry. `default: false` preserves the pre-facade
        // per-platform mapping (the macOS struct never had the flag).
        Ok(super::macos_loopback::list_loopback_sources()?
            .into_iter()
            .map(|s| LoopbackDevice {
                id: s.id,
                label: s.label,
                default: false,
            })
            .collect())
    }
    #[cfg(target_os = "windows")]
    {
        Ok(super::windows_loopback::list_loopback_sources()?
            .into_iter()
            .map(|s| LoopbackDevice {
                id: s.id,
                label: s.label,
                default: s.default,
            })
            .collect())
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Ok(Vec::new())
    }
}

/// Construct the loopback source for `id`.
///
/// Error contract: an unknown id yields a message containing "not found" —
/// the shell's StartError::classify keys `device-missing` off that substring.
pub fn by_id(id: &str) -> Result<Box<dyn AudioSource>> {
    #[cfg(target_os = "macos")]
    {
        use super::macos_loopback::{list_loopback_sources, MacLoopbackSource};
        let target = list_loopback_sources()?
            .into_iter()
            .find(|s| s.id == id)
            .ok_or_else(|| crate::Error::Audio(format!("loopback source not found: {id}")))?;
        Ok(Box::new(MacLoopbackSource::new(target)))
    }
    #[cfg(target_os = "windows")]
    {
        // Windows resolves the endpoint lazily at start(); an unknown id
        // surfaces there as "render device not found: {id}".
        Ok(Box::new(super::windows_loopback::WinLoopbackSource::by_id(
            id,
        )))
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = id;
        Err(crate::Error::Audio(
            "loopback unsupported on this platform".into(),
        ))
    }
}
