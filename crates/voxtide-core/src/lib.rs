//! voxtide-core — audio capture, Soniox client, persistence, keychain.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod audio;
pub mod config;
mod error;
pub mod keychain;
pub mod latency;
pub mod persistence;
pub mod session;
pub mod speaker_map;
pub mod translation;

pub use error::{Error, Result};
pub use keychain::Keychain;

/// Wall-clock milliseconds since the Unix epoch. Saturates to 0 on clock
/// skew (pre-1970) so callers never panic. The single source of "now" for
/// session rows and token timestamps — keep it that way (two drifting copies
/// is how the epoch/relative ts_ms mix-up started).
pub(crate) fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
