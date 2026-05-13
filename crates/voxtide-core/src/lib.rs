//! voxtide-core — audio capture, Soniox client, persistence, keychain.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

mod error;
pub mod audio;
pub mod translation;
pub mod speaker_map;
pub mod latency;

pub use error::{Error, Result};
