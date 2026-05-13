//! voxtide-core — audio capture, Soniox client, persistence, keychain.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

mod error;
pub mod audio;
pub mod config;
pub mod translation;
pub mod speaker_map;
pub mod latency;
pub mod persistence;
pub mod keychain;

pub use error::{Error, Result};
pub use keychain::Keychain;
