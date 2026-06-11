//! Shared cpal capture scaffold: one copy of the capture-thread lifecycle,
//! the per-sample-format stream dispatch, and the resample→i16→chunk→send
//! pipeline. Used by the microphone source (all platforms) and the WASAPI
//! loopback source (Windows); the ScreenCaptureKit source reuses just the
//! [`CapturePipeline`] half (its thread model is SCKit-specific).
//!
//! Previously mic.rs and windows_loopback.rs were near-verbatim copies of
//! each other (with quietly drifting comments) and macos_loopback.rs carried
//! a third inline copy of the sample pipeline.

use cpal::traits::{DeviceTrait, StreamTrait};
use tokio::sync::mpsc;

use crate::audio::resampler::{f32_to_i16, Resampler, ResamplerSpec};
use crate::audio::{channel, AudioFrame, AudioStream, Chunker, INIT_TIMEOUT};
use crate::{Error, Result};

/// Resample → i16-convert → chunk → try_send, with every intermediate buffer
/// preallocated and reused across callbacks. Audio data callbacks run on a
/// real-time thread: the old per-callback `iter().map(..).collect()` chains
/// allocated three fresh `Vec`s per device period, risking priority-inversion
/// stalls under allocator contention. The scratch buffers here are long-lived;
/// steady-state callbacks allocate only when the chunker emits a full
/// [`AudioFrame`] (10×/s, owned by the channel consumer).
pub struct CapturePipeline {
    resampler: Resampler,
    chunker: Chunker,
    /// Sample-format conversion target (`push_converted`).
    f32_conv: Vec<f32>,
    /// Resampler output.
    f32_out: Vec<f32>,
    /// i16 conversion target fed to the chunker.
    i16_out: Vec<i16>,
    tx: mpsc::Sender<AudioFrame>,
    /// Log label ("mic", "wasapi", "sckit").
    label: &'static str,
}

impl CapturePipeline {
    pub fn new(resampler: Resampler, tx: mpsc::Sender<AudioFrame>, label: &'static str) -> Self {
        Self {
            resampler,
            chunker: Chunker::new(),
            f32_conv: Vec::new(),
            f32_out: Vec::new(),
            i16_out: Vec::new(),
            tx,
            label,
        }
    }

    /// Push interleaved f32 samples through resample → i16 → chunk → send.
    pub fn push_f32(&mut self, samples: &[f32]) {
        if let Err(e) = self.resampler.process_into(samples, &mut self.f32_out) {
            tracing::error!(?e, label = self.label, "resample");
            return;
        }
        self.i16_out.clear();
        self.i16_out.reserve(self.f32_out.len());
        self.i16_out
            .extend(self.f32_out.iter().copied().map(f32_to_i16));
        for f in self.chunker.push(&self.i16_out) {
            if self.tx.try_send(f).is_err() {
                tracing::warn!(label = self.label, "backpressure: dropping frame");
            }
        }
    }

    /// Convert a non-f32 callback buffer into the f32 scratch, then push.
    pub fn push_converted<T: Copy>(&mut self, data: &[T], to_f32: impl Fn(T) -> f32) {
        // mem::take so filling the scratch and `push_f32(&conv)` don't alias
        // `self`; the buffer is handed back afterwards, so no allocation.
        let mut conv = std::mem::take(&mut self.f32_conv);
        conv.clear();
        conv.reserve(data.len());
        conv.extend(data.iter().copied().map(to_f32));
        self.push_f32(&conv);
        self.f32_conv = conv;
    }
}

/// Picks the device and its stream config for a cpal source. Runs ON the
/// capture thread so the !Send cpal types never cross a thread boundary.
/// Input sources use `default_input_config`; WASAPI loopback captures a
/// *render* endpoint, so it uses `default_output_config`.
pub(crate) type OpenFn =
    Box<dyn FnOnce(&cpal::Host) -> Result<(cpal::Device, cpal::SupportedStreamConfig)> + Send>;

/// What a cpal-backed source must provide; [`start_capture`] owns the rest.
pub(crate) struct CpalCaptureSpec {
    /// Capture-thread name (shows up in debuggers/profilers).
    pub thread_name: String,
    /// Log + error-message label ("mic", "wasapi").
    pub label: &'static str,
    pub open: OpenFn,
}

/// Spawn the capture thread and run the full cpal source lifecycle:
/// open device → build stream for its sample format → play → signal init →
/// wait for stop/error → drop the stream (closing the frame channel).
pub(crate) fn start_capture(spec: CpalCaptureSpec) -> Result<AudioStream> {
    let (tx, rx) = channel();
    let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel::<()>();

    // Synchronous init-error channel: the spawned thread sends Ok(()) once
    // the stream is playing, or Err(...) if setup fails.
    let (init_tx, init_rx) = std::sync::mpsc::sync_channel::<Result<()>>(1);

    let label = spec.label;
    let open = spec.open;

    std::thread::Builder::new()
        .name(spec.thread_name)
        .spawn(move || {
            // Everything cpal-related lives entirely on this thread so that
            // the !Send cpal::Stream never crosses a thread boundary.
            let host = cpal::default_host();
            let (device, supported_config) = match open(&host) {
                Ok(v) => v,
                Err(e) => {
                    let _ = init_tx.send(Err(e));
                    return;
                }
            };

            let sample_rate = supported_config.sample_rate().0;
            let channels = supported_config.channels();
            let sample_format = supported_config.sample_format();
            // Convert to StreamConfig before the match so the supported config
            // isn't consumed inside multiple match arms.
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
            let pipeline = CapturePipeline::new(resampler, tx.clone(), label);

            // Stream-error signal: cpal invokes the error callback when the
            // device/endpoint is lost (unplugged, default switch, HAL/WASAPI
            // reset). The callback fires `err_tx`; the wait loop below wakes
            // on it and drops the stream, which closes the frame channel so
            // the session worker can finalize instead of idling forever on a
            // now-dead device.
            //
            // Stop and error are deliberately TWO channels, not one: `stop` is
            // a tokio oneshot because it crosses into [`AudioStream`]'s public
            // API (the session worker holds the sender), while the error
            // signal is a std mpsc local to this thread because the RT
            // callback must never touch an async runtime. Folding them
            // together would need an adapter task for zero gain — the 200 ms
            // poll below already multiplexes both.
            let (err_tx, err_rx) = std::sync::mpsc::channel::<()>();

            // One stream per sample-format arm. `pipeline` is moved into the
            // data closure — plain ownership, no Arc/Mutex. The macro expands
            // exactly once per match arm, so each arm gets sole ownership.
            macro_rules! build_stream {
                ($sample_ty:ty, $to_f32:expr) => {{
                    let mut pipe_cb = pipeline;
                    let err_tx_cb = err_tx.clone();
                    device
                        .build_input_stream(
                            &stream_config,
                            move |data: &[$sample_ty], _| pipe_cb.push_converted(data, $to_f32),
                            move |e| {
                                tracing::error!(?e, label, "cpal stream error");
                                let _ = err_tx_cb.send(());
                            },
                            None,
                        )
                        .map_err(|e| Error::Audio(format!("cpal build_input_stream: {e}")))
                }};
            }

            let stream_result: Result<cpal::Stream> = match sample_format {
                cpal::SampleFormat::F32 => build_stream!(f32, |s| s),
                cpal::SampleFormat::F64 => build_stream!(f64, |s| s as f32),
                cpal::SampleFormat::I8 => build_stream!(i8, |s| s as f32 / i8::MAX as f32),
                cpal::SampleFormat::I16 => build_stream!(i16, |s| s as f32 / i16::MAX as f32),
                cpal::SampleFormat::I32 => build_stream!(i32, |s| s as f32 / i32::MAX as f32),
                cpal::SampleFormat::I64 => build_stream!(i64, |s| s as f32 / i64::MAX as f32),
                cpal::SampleFormat::U8 => build_stream!(u8, |s| (s as f32 - 128.0) / 128.0),
                cpal::SampleFormat::U16 => {
                    build_stream!(u16, |s| (s as f32 - 32768.0) / 32768.0)
                }
                cpal::SampleFormat::U32 => {
                    build_stream!(u32, |s| {
                        (s as f64 - 2_147_483_648.0_f64) as f32 / 2_147_483_648.0_f32
                    })
                }
                cpal::SampleFormat::U64 => {
                    build_stream!(u64, |s| (s as f64 / u64::MAX as f64 * 2.0 - 1.0) as f32)
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
                let _ = init_tx.send(Err(Error::Audio(format!("{label} play: {e}"))));
                return;
            }

            // Signal the parent thread that init succeeded.
            let _ = init_tx.send(Ok(()));

            // Wait for an explicit stop OR a stream error, whichever comes
            // first. `stop_rx` is a tokio oneshot but we only ever poll it
            // synchronously here (no runtime needed): a short `err_rx` timeout
            // bounds how long a `try_recv` of the stop signal is delayed.
            //
            // - err_rx Ok      → the device was lost (callback fired). Break,
            //   drop the stream, close the frame channel; the worker finalizes.
            // - err_rx Disconnected → every error-tx clone dropped (cannot
            //   happen while the stream lives, since its callback holds one):
            //   treat as terminal and break.
            // - stop_rx Ok/Closed → explicit stop, or the stop sender was
            //   dropped (the init-timeout path drops it). Break.
            // - stop_rx Empty   → neither yet; keep waiting.
            loop {
                match err_rx.recv_timeout(std::time::Duration::from_millis(200)) {
                    Ok(()) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => match stop_rx.try_recv() {
                        Err(tokio::sync::oneshot::error::TryRecvError::Empty) => continue,
                        _ => break,
                    },
                }
            }

            // Dropping `stream` here stops the capture and drops the data
            // callback's pipeline (with its `tx` clone). The original `tx` in
            // this thread scope drops when the closure returns just below, so
            // once both are gone the frame channel closes and the session
            // worker's audio arm sees `None`.
            drop(stream);
        })
        .map_err(|e| Error::Audio(format!("{label} thread spawn: {e}")))?;

    // Wait for the spawned thread to confirm init succeeded or failed, but
    // never block start() indefinitely: a wedged init (TCC dialog, HAL wedge)
    // would otherwise park the calling tokio worker forever and leave the
    // controller's slot stuck Pending, which stop() cannot clear. On timeout,
    // signal the capture thread to halt by dropping the stop sender (its wait
    // loop's `stop_rx.try_recv()` then returns `Closed` and breaks once it
    // reaches that point, so it does not linger holding the device) and
    // surface a timeout error.
    match init_rx.recv_timeout(INIT_TIMEOUT) {
        Ok(Ok(())) => {}
        Ok(Err(e)) => return Err(e),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            drop(stop_tx);
            return Err(Error::Audio(format!(
                "{label} audio init timed out ({}s)",
                INIT_TIMEOUT.as_secs()
            )));
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            return Err(Error::Audio(format!(
                "{label} thread terminated before signalling init"
            )))
        }
    }

    Ok(AudioStream { rx, stop: stop_tx })
}
