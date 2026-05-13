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
