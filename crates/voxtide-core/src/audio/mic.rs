use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use parking_lot::Mutex;
use tokio::sync::{mpsc, oneshot};

use crate::audio::resampler::{f32_to_i16, Resampler, ResamplerSpec};
use crate::audio::{channel, AudioFrame, AudioSource, AudioStream, Chunker, SourceKind};
use crate::{Error, Result};

// ─── Device listing ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MicDevice {
    pub id: String,
    pub label: String,
    pub default: bool,
}

pub fn list_input_devices() -> Result<Vec<MicDevice>> {
    let host = cpal::default_host();
    let default = host.default_input_device().and_then(|d| d.name().ok());
    let devices = host
        .input_devices()
        .map_err(|e| Error::Audio(format!("cpal input_devices: {e}")))?;
    let mut out = Vec::new();
    for d in devices {
        let name = d
            .name()
            .map_err(|e| Error::Audio(format!("cpal name: {e}")))?;
        out.push(MicDevice {
            id: name.clone(),
            default: default.as_deref() == Some(&name),
            label: name,
        });
    }
    Ok(out)
}

// ─── MicSource ───────────────────────────────────────────────────────────────

pub struct MicSource {
    device_id: Option<String>,
    label: String,
}

impl MicSource {
    pub fn default_device() -> Result<Self> {
        Ok(Self {
            device_id: None,
            label: "Default microphone".into(),
        })
    }

    pub fn by_id(id: &str) -> Self {
        Self {
            device_id: Some(id.to_string()),
            label: id.to_string(),
        }
    }
}

// ─── Internal helpers ────────────────────────────────────────────────────────

fn pick_device(host: &cpal::Host, want_id: Option<&str>) -> Result<cpal::Device> {
    if let Some(id) = want_id {
        for d in host
            .input_devices()
            .map_err(|e| Error::Audio(e.to_string()))?
        {
            if d.name().ok().as_deref() == Some(id) {
                return Ok(d);
            }
        }
        return Err(Error::Audio(format!("mic device not found: {id}")));
    }
    host.default_input_device()
        .ok_or_else(|| Error::Audio("no default input device".into()))
}

/// Convert a &[f32] slice into AudioFrames and push them onto the channel.
fn push_f32_samples(
    samples: &[f32],
    resampler: &Arc<Mutex<Resampler>>,
    chunker: &Arc<Mutex<Chunker>>,
    tx: &mpsc::Sender<AudioFrame>,
) {
    let processed = match resampler.lock().process(samples) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(?e, "mic resample");
            return;
        }
    };
    let i16s: Vec<i16> = processed.into_iter().map(f32_to_i16).collect();
    let frames: Vec<AudioFrame> = chunker.lock().push(&i16s).collect();
    for f in frames {
        if tx.try_send(f).is_err() {
            tracing::warn!("mic backpressure: dropping frame");
        }
    }
}

// ─── AudioSource impl ────────────────────────────────────────────────────────

impl AudioSource for MicSource {
    fn start(&self) -> Result<AudioStream> {
        let (tx, rx) = channel();
        let (stop_tx, stop_rx) = oneshot::channel::<()>();

        // Synchronous init-error channel: the spawned thread sends Ok(()) once
        // the stream is playing, or Err(...) if setup fails.
        let (init_tx, init_rx) = std::sync::mpsc::sync_channel::<Result<()>>(1);

        let device_id = self.device_id.clone();
        let thread_label = self.label.clone();

        std::thread::Builder::new()
            .name(format!("mic-{}", thread_label))
            .spawn(move || {
                // Everything cpal-related lives entirely on this thread so that
                // the !Send cpal::Stream never crosses a thread boundary.

                let host = cpal::default_host();
                let device = match pick_device(&host, device_id.as_deref()) {
                    Ok(d) => d,
                    Err(e) => {
                        let _ = init_tx.send(Err(e));
                        return;
                    }
                };

                let supported_config = match device.default_input_config() {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = init_tx.send(Err(Error::Audio(format!(
                            "cpal default_input_config: {e}"
                        ))));
                        return;
                    }
                };

                let sample_rate = supported_config.sample_rate().0;
                let channels = supported_config.channels();
                let sample_format = supported_config.sample_format();
                // Convert to StreamConfig before the match so `config` isn't
                // consumed inside multiple match arms.
                let stream_config: cpal::StreamConfig = supported_config.into();

                let resampler = match Resampler::new(ResamplerSpec {
                    source_hz: sample_rate,
                    source_channels: channels,
                }) {
                    Ok(r) => Arc::new(Mutex::new(r)),
                    Err(e) => {
                        let _ = init_tx.send(Err(e));
                        return;
                    }
                };
                let chunker = Arc::new(Mutex::new(Chunker::new()));

                // Macro-like helper to reduce repetition per sample format arm.
                macro_rules! build_stream {
                    ($sample_ty:ty, $to_f32:expr) => {{
                        let tx_cb = tx.clone();
                        let r_cb = resampler.clone();
                        let c_cb = chunker.clone();
                        device.build_input_stream(
                            &stream_config,
                            move |data: &[$sample_ty], _| {
                                let f32s: Vec<f32> = data.iter().map($to_f32).collect();
                                push_f32_samples(&f32s, &r_cb, &c_cb, &tx_cb);
                            },
                            |e| tracing::error!(?e, "cpal stream error"),
                            None,
                        )
                        .map_err(|e| Error::Audio(format!("cpal build_input_stream: {e}")))
                    }};
                }

                let stream_result: Result<cpal::Stream> = match sample_format {
                    cpal::SampleFormat::F32 => build_stream!(f32, |&s| s),
                    cpal::SampleFormat::F64 => {
                        build_stream!(f64, |&s| s as f32)
                    }
                    cpal::SampleFormat::I8 => {
                        build_stream!(i8, |&s| s as f32 / i8::MAX as f32)
                    }
                    cpal::SampleFormat::I16 => {
                        build_stream!(i16, |&s| s as f32 / i16::MAX as f32)
                    }
                    cpal::SampleFormat::I32 => {
                        build_stream!(i32, |&s| s as f32 / i32::MAX as f32)
                    }
                    cpal::SampleFormat::I64 => {
                        build_stream!(i64, |&s| s as f32 / i64::MAX as f32)
                    }
                    cpal::SampleFormat::U8 => {
                        build_stream!(u8, |&s| (s as f32 - 128.0) / 128.0)
                    }
                    cpal::SampleFormat::U16 => {
                        build_stream!(u16, |&s| (s as f32 - 32768.0) / 32768.0)
                    }
                    cpal::SampleFormat::U32 => {
                        build_stream!(u32, |&s| {
                            (s as f64 - 2_147_483_648.0_f64) as f32 / 2_147_483_648.0_f32
                        })
                    }
                    cpal::SampleFormat::U64 => {
                        build_stream!(u64, |&s| {
                            (s as f64 / u64::MAX as f64 * 2.0 - 1.0) as f32
                        })
                    }
                    other => Err(Error::Audio(format!(
                        "unsupported sample format: {other:?}"
                    ))),
                };

                let stream = match stream_result {
                    Ok(s) => s,
                    Err(e) => {
                        let _ = init_tx.send(Err(e));
                        return;
                    }
                };

                if let Err(e) = stream.play() {
                    let _ =
                        init_tx.send(Err(Error::Audio(format!("cpal play: {e}"))));
                    return;
                }

                // Signal the parent thread that init succeeded.
                let _ = init_tx.send(Ok(()));

                // Block this thread until the stop signal arrives or the sender
                // is dropped (caller dropped the AudioStream).
                // We build a minimal single-threaded Tokio runtime just for
                // this blocking wait — futures_util::executor is not re-exported
                // and adding the `futures` crate is unnecessary overhead.
                let _ = tokio::runtime::Builder::new_current_thread()
                    .build()
                    .expect("mic stop-wait runtime")
                    .block_on(stop_rx);

                // Dropping `stream` here stops the capture.
                drop(stream);
            })
            .map_err(|e| Error::Audio(format!("mic thread spawn: {e}")))?;

        // Wait for the spawned thread to confirm init succeeded or failed.
        match init_rx.recv() {
            Ok(Ok(())) => {}
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                return Err(Error::Audio(
                    "mic thread terminated before signalling init".into(),
                ))
            }
        }

        Ok(AudioStream { rx, stop: stop_tx })
    }

    fn label(&self) -> &str {
        &self.label
    }

    fn kind(&self) -> SourceKind {
        SourceKind::Mic
    }
}
