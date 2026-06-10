#![cfg(target_os = "windows")]
//! Windows system loopback audio capture via WASAPI render endpoint.
//!
//! cpal 0.15 exposes WASAPI loopback by calling `build_input_stream` on a
//! device obtained from `host.default_output_device()` (or any output device).
//! Windows internally routes this as a WASAPI loopback capture stream.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use tokio::sync::{mpsc, oneshot};

use crate::audio::resampler::{f32_to_i16, Resampler, ResamplerSpec};
use crate::audio::{channel, AudioFrame, AudioSource, AudioStream, Chunker, SourceKind};
use crate::{Error, Result};

// ─── Device listing ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoopbackSource {
    pub id: String,
    pub label: String,
    pub default: bool,
}

/// List all available render (output) endpoints that can be used as loopback sources.
pub fn list_loopback_sources() -> Result<Vec<LoopbackSource>> {
    let host = cpal::default_host();
    let default_name = host.default_output_device().and_then(|d| d.name().ok());
    let mut out = Vec::new();
    for d in host
        .output_devices()
        .map_err(|e| Error::Audio(format!("cpal output_devices: {e}")))?
    {
        let name = d
            .name()
            .map_err(|e| Error::Audio(format!("cpal device name: {e}")))?;
        out.push(LoopbackSource {
            default: default_name.as_deref() == Some(&name),
            label: format!("{name} (loopback)"),
            id: name,
        });
    }
    Ok(out)
}

// ─── WinLoopbackSource ────────────────────────────────────────────────────────

pub struct WinLoopbackSource {
    device_id: Option<String>,
    label: String,
}

impl WinLoopbackSource {
    /// Capture from the system default render (output) endpoint.
    pub fn default_render() -> Self {
        Self {
            device_id: None,
            label: "Default render (loopback)".into(),
        }
    }

    /// Capture from the render endpoint identified by `id`.
    pub fn by_id(id: &str) -> Self {
        Self {
            device_id: Some(id.to_string()),
            label: format!("{id} (loopback)"),
        }
    }
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

fn pick_render_device(host: &cpal::Host, want_id: Option<&str>) -> Result<cpal::Device> {
    if let Some(id) = want_id {
        for d in host
            .output_devices()
            .map_err(|e| Error::Audio(e.to_string()))?
        {
            if d.name().ok().as_deref() == Some(id) {
                return Ok(d);
            }
        }
        return Err(Error::Audio(format!("render device not found: {id}")));
    }
    host.default_output_device()
        .ok_or_else(|| Error::Audio("no default render device".into()))
}

/// Convert a `&[f32]` slice into AudioFrames and push them onto the channel.
fn push_f32_samples(
    samples: &[f32],
    resampler: &mut Resampler,
    chunker: &mut Chunker,
    tx: &mpsc::Sender<AudioFrame>,
) {
    let processed = match resampler.process(samples) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(?e, "wasapi resample");
            return;
        }
    };
    let i16s: Vec<i16> = processed.into_iter().map(f32_to_i16).collect();
    let frames: Vec<AudioFrame> = chunker.push(&i16s).collect();
    for f in frames {
        if tx.try_send(f).is_err() {
            tracing::warn!("wasapi backpressure: dropping frame");
        }
    }
}

// ─── AudioSource impl ─────────────────────────────────────────────────────────

impl AudioSource for WinLoopbackSource {
    fn start(&self) -> Result<AudioStream> {
        let (tx, rx) = channel();
        let (stop_tx, stop_rx) = oneshot::channel::<()>();

        // Synchronous init-error channel: the spawned thread sends Ok(()) once
        // the stream is playing, or Err(...) if setup fails.
        let (init_tx, init_rx) = std::sync::mpsc::sync_channel::<Result<()>>(1);

        let device_id = self.device_id.clone();
        let thread_label = self.label.clone();

        std::thread::Builder::new()
            .name(format!("wasapi-loopback-{}", thread_label))
            .spawn(move || {
                // Everything cpal-related lives entirely on this thread so that
                // the !Send cpal::Stream never crosses a thread boundary.

                let host = cpal::default_host();
                let device = match pick_render_device(&host, device_id.as_deref()) {
                    Ok(d) => d,
                    Err(e) => {
                        let _ = init_tx.send(Err(e));
                        return;
                    }
                };

                // Use the default *output* config — cpal/WASAPI loopback inherits
                // the render endpoint's native format.
                let supported_config = match device.default_output_config() {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = init_tx.send(Err(Error::Audio(format!(
                            "cpal default_output_config: {e}"
                        ))));
                        return;
                    }
                };

                let sample_rate = supported_config.sample_rate().0;
                let channels = supported_config.channels();
                let sample_format = supported_config.sample_format();
                // Extract StreamConfig once so it is not consumed inside the match arms.
                let stream_config: cpal::StreamConfig = supported_config.into();

                let resampler = match Resampler::new(ResamplerSpec {
                    source_hz: sample_rate,
                    source_channels: channels,
                }) {
                    Ok(r) => r,
                    Err(e) => {
                        let _ = init_tx.send(Err(e));
                        return;
                    }
                };
                let chunker = Chunker::new();

                // Macro: build one input-stream per sample format arm.
                // `resampler` and `chunker` are moved into exactly one closure — plain
                // ownership, no Arc/Mutex needed.
                macro_rules! build_stream {
                    ($sample_ty:ty, $to_f32:expr) => {{
                        let tx_cb = tx.clone();
                        let mut r_cb = resampler;
                        let mut c_cb = chunker;
                        device
                            .build_input_stream(
                                &stream_config,
                                move |data: &[$sample_ty], _| {
                                    let f32s: Vec<f32> = data.iter().map($to_f32).collect();
                                    push_f32_samples(&f32s, &mut r_cb, &mut c_cb, &tx_cb);
                                },
                                |e| tracing::error!(?e, "wasapi stream error"),
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
                    let _ = init_tx.send(Err(Error::Audio(format!("wasapi play: {e}"))));
                    return;
                }

                // Build the stop-wait runtime BEFORE signalling init success so
                // that a build failure is propagated to the caller rather than
                // silently panicking after Ok(()) was already sent.
                let rt = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(r) => r,
                    Err(e) => {
                        let _ = init_tx
                            .send(Err(Error::Audio(format!("wasapi stop-wait runtime: {e}"))));
                        return;
                    }
                };

                // Signal the parent thread that init succeeded.
                let _ = init_tx.send(Ok(()));

                // Block this thread until the stop signal arrives or the sender
                // is dropped (caller dropped the AudioStream).
                let _ = rt.block_on(stop_rx);

                // Dropping `stream` here stops the WASAPI loopback capture.
                drop(stream);
            })
            .map_err(|e| Error::Audio(format!("wasapi thread spawn: {e}")))?;

        // Wait for the spawned thread to confirm init succeeded or failed, but never block
        // start() indefinitely: a wedged init would otherwise park the calling tokio worker
        // forever and leave the controller's slot stuck Pending, which stop() cannot clear. On
        // timeout, signal the capture thread to halt by dropping the stop sender (it wakes the
        // thread's `block_on(stop_rx)` once it reaches that point so it does not linger holding
        // the endpoint) and surface a timeout error.
        match init_rx.recv_timeout(std::time::Duration::from_secs(10)) {
            Ok(Ok(())) => {}
            Ok(Err(e)) => return Err(e),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                drop(stop_tx);
                return Err(Error::Audio("wasapi audio init timed out".into()));
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                return Err(Error::Audio(
                    "wasapi thread terminated before signalling init".into(),
                ))
            }
        }

        Ok(AudioStream { rx, stop: stop_tx })
    }

    fn label(&self) -> &str {
        &self.label
    }

    fn kind(&self) -> SourceKind {
        SourceKind::SystemLoopback
    }
}
