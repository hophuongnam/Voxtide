use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;
use tokio::sync::oneshot;

use crate::audio::resampler::{f32_to_i16, Resampler, ResamplerSpec};
use crate::audio::{channel, AudioSource, AudioStream, Chunker, SourceKind, CHUNK_MS};
use crate::{Error, Result};

/// Interleaved PCM data together with its source format, shared between open() and the async task.
type WavData = Arc<Mutex<Option<(u32, u16, Vec<f32>)>>>;

pub struct WavSource {
    path: PathBuf,
    realtime: bool,
    label: String,
    samples: WavData,
}

impl WavSource {
    pub fn open(path: &Path, realtime: bool) -> Result<Self> {
        let mut reader = hound::WavReader::open(path)
            .map_err(|e| Error::Audio(format!("wav open: {e}")))?;
        let spec = reader.spec();
        let mut samples: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Int => {
                let max = ((1i64 << (spec.bits_per_sample - 1)) - 1) as f32;
                reader.samples::<i32>()
                    .map(|s| s.map(|v| v as f32 / max).map_err(|e| Error::Audio(format!("wav read: {e}"))))
                    .collect::<Result<_>>()?
            }
            // IEEE_FLOAT WAVs are standardised in [-1.0, 1.0]; no normalisation needed.
            hound::SampleFormat::Float => {
                reader.samples::<f32>()
                    .map(|s| s.map_err(|e| Error::Audio(format!("wav read: {e}"))))
                    .collect::<Result<_>>()?
            }
        };
        samples.shrink_to_fit();
        let label = path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "wav".into());
        Ok(Self {
            path: path.to_path_buf(),
            realtime,
            label,
            samples: Arc::new(Mutex::new(Some((spec.sample_rate, spec.channels, samples)))),
        })
    }
}

impl AudioSource for WavSource {
    fn start(&self) -> Result<AudioStream> {
        let (tx, rx) = channel();
        let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
        let realtime = self.realtime;
        let samples_arc = self.samples.clone();
        let path = self.path.clone();

        tokio::spawn(async move {
            let (rate, ch, all) = match samples_arc.lock().take() {
                Some(v) => v,
                None => {
                    tracing::warn!(?path, "WavSource already drained");
                    return;
                }
            };
            let mut resampler = match Resampler::new(ResamplerSpec { source_hz: rate, source_channels: ch }) {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!(?e, "resampler init failed");
                    return;
                }
            };
            let mut chunker = Chunker::new();

            // Feed roughly 100 ms of source audio per loop tick.
            let frames_per_tick = (rate as usize * ch as usize * CHUNK_MS as usize) / 1000;
            let mut cursor = 0usize;
            let interval = Duration::from_millis(CHUNK_MS as u64);
            loop {
                match stop_rx.try_recv() {
                    Ok(_) | Err(tokio::sync::oneshot::error::TryRecvError::Closed) => break,
                    Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {}
                }
                if cursor >= all.len() { break; }
                let end = (cursor + frames_per_tick).min(all.len());
                let processed = match resampler.process(&all[cursor..end]) {
                    Ok(v) => v,
                    Err(e) => { tracing::error!(?e, "resample"); break; }
                };
                let i16s: Vec<i16> = processed.into_iter().map(f32_to_i16).collect();
                for frame in chunker.push(&i16s) {
                    if tx.send(frame).await.is_err() { return; }
                }
                cursor = end;
                if realtime { tokio::time::sleep(interval).await; }
            }
            // Any sub-frame remainder in chunker.buf is intentionally discarded;
            // callers must not rely on receiving the final partial frame.
        });

        Ok(AudioStream { rx, stop: stop_tx })
    }
    fn label(&self) -> &str { &self.label }
    fn kind(&self) -> SourceKind { SourceKind::Mock }
}
