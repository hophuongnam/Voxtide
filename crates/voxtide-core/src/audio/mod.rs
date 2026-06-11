//! Audio capture abstraction. Output is always 16 kHz mono s16le PCM in 1600-sample (100 ms) chunks.

pub mod cpal_pipeline;
pub mod mock;
pub mod resampler;

#[cfg(target_os = "macos")]
pub mod macos_loopback;
pub mod mic;
#[cfg(target_os = "windows")]
pub mod windows_loopback;

use tokio::sync::{mpsc, oneshot};

use crate::Result;

pub const SAMPLE_RATE_HZ: u32 = 16_000;
pub const CHANNELS: u16 = 1;
pub const CHUNK_MS: u32 = 100;
pub const CHUNK_SAMPLES: usize = (SAMPLE_RATE_HZ as usize * CHUNK_MS as usize) / 1000;

/// How long a source's `start()` waits for its capture thread to signal init
/// success before giving up (TCC permission dialog, HAL/SCKit wedge). Shared
/// by every capture backend so the timeout — and the "(Ns)" in the error
/// message — can never drift between them.
pub(crate) const INIT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

#[derive(Debug, Clone)]
pub struct AudioFrame {
    pub samples: Vec<i16>,
}

impl AudioFrame {
    pub fn to_le_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.samples.len() * 2);
        for s in &self.samples {
            out.extend_from_slice(&s.to_le_bytes());
        }
        out
    }
}

#[derive(Default)]
pub struct Chunker {
    buf: Vec<i16>,
}

impl Chunker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push<'a>(&'a mut self, input: &[i16]) -> impl Iterator<Item = AudioFrame> + 'a {
        self.buf.extend_from_slice(input);
        std::iter::from_fn(move || {
            if self.buf.len() < CHUNK_SAMPLES {
                return None;
            }
            let chunk: Vec<i16> = self.buf.drain(..CHUNK_SAMPLES).collect();
            Some(AudioFrame { samples: chunk })
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceKind {
    Mic,
    SystemLoopback,
    Mock,
}

pub struct AudioStream {
    pub rx: mpsc::Receiver<AudioFrame>,
    pub stop: oneshot::Sender<()>,
}

pub trait AudioSource: Send + Sync {
    fn start(&self) -> Result<AudioStream>;
    fn label(&self) -> &str;
    fn kind(&self) -> SourceKind;
}

pub fn channel() -> (mpsc::Sender<AudioFrame>, mpsc::Receiver<AudioFrame>) {
    mpsc::channel(64)
}

/// Interleave per-channel (planar) buffers: `[[L...],[R...]] -> [L0,R0,L1,R1,...]`.
/// A single buffer is returned as-is (already interleaved or mono).
/// Uneven channel lengths truncate to the shortest.
pub fn planar_to_interleaved(channels: &[Vec<f32>]) -> Vec<f32> {
    let mut out = Vec::new();
    planar_to_interleaved_into(channels, &mut out);
    out
}

/// [`planar_to_interleaved`] into a caller-owned buffer (cleared first), so
/// real-time capture callbacks can reuse one scratch allocation per callback
/// instead of allocating a fresh `Vec` each time.
pub fn planar_to_interleaved_into(channels: &[Vec<f32>], out: &mut Vec<f32>) {
    out.clear();
    if channels.len() == 1 {
        out.extend_from_slice(&channels[0]);
        return;
    }
    let frames = channels.iter().map(|c| c.len()).min().unwrap_or(0);
    out.reserve(frames * channels.len());
    for i in 0..frames {
        for ch in channels {
            out.push(ch[i]);
        }
    }
}
