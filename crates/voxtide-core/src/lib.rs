//! voxtide-core — audio capture, Soniox client, persistence, keychain.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

mod error;
pub use error::{Error, Result};
