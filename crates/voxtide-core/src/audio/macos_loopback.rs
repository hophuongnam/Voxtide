#![cfg(target_os = "macos")]
//! macOS system loopback audio capture via ScreenCaptureKit.
//!
//! v1 always captures via `ScreenCaptureKit` (macOS 13.0+), which covers
//! Meeting mode end-to-end.  The `CaptureStrategy` enum and `capture_strategy()`
//! helper are kept so a future Task (v1.1) can add the Core-Audio ProcessTap
//! path (macOS 14.4+) without changing callers.

use parking_lot::Mutex;
use tokio::sync::oneshot;

use crate::audio::cpal_pipeline::CapturePipeline;
use crate::audio::resampler::{Resampler, ResamplerSpec};
use crate::audio::{channel, AudioSource, AudioStream, INIT_TIMEOUT};
use crate::{Error, Result};

// ─── Strategy selector ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureStrategy {
    /// macOS 14.4+ — Core Audio `AudioHardwareCreateProcessTap`.
    /// Not wired in v1; reserved for the v1.1 follow-up task.
    ProcessTap,
    /// macOS 13.0+ — ScreenCaptureKit render endpoint.  The v1 default.
    ScreenCaptureKit,
}

/// Return the capture strategy that should be used on the current OS.
///
/// v1 always returns [`CaptureStrategy::ScreenCaptureKit`].  The ProcessTap
/// branch in a future task (v1.1) can change the return logic here without
/// touching callers.
pub fn capture_strategy() -> CaptureStrategy {
    // v1 always returns ScreenCaptureKit. The macOS-version probe is kept for the v1.1 path
    // when ProcessTap lands. We do not branch on it yet because the FFI handshake is out of
    // scope for this plan and ScreenCaptureKit already covers Meeting mode on 13.0+.
    CaptureStrategy::ScreenCaptureKit
}

// ─── Device listing ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoopbackSource {
    pub id: String,
    pub label: String,
    /// `true` when the endpoint is actively producing audio.
    pub running: bool,
}

/// Return the available loopback sources.
///
/// ScreenCaptureKit captures the entire system render endpoint; we therefore
/// expose a single synthetic entry.
pub fn list_loopback_sources() -> Result<Vec<LoopbackSource>> {
    Ok(vec![LoopbackSource {
        id: "system".into(),
        label: "System audio (default output)".into(),
        running: true,
    }])
}

// ─── MacLoopbackSource ───────────────────────────────────────────────────────

pub struct MacLoopbackSource {
    target: LoopbackSource,
}

impl MacLoopbackSource {
    pub fn new(target: LoopbackSource) -> Self {
        Self { target }
    }
}

impl AudioSource for MacLoopbackSource {
    fn start(&self) -> Result<AudioStream> {
        sckit::start()
    }

    fn label(&self) -> &str {
        &self.target.label
    }
}

// ─── ScreenCaptureKit implementation ─────────────────────────────────────────
mod sckit {
    use super::*;

    use screencapturekit::{
        output::CMSampleBuffer,
        shareable_content::SCShareableContent,
        stream::{
            configuration::SCStreamConfiguration, content_filter::SCContentFilter,
            output_trait::SCStreamOutputTrait, output_type::SCStreamOutputType, SCStream,
        },
    };

    // ── Output handler ────────────────────────────────────────────────────────

    struct OutputHandler {
        /// All mutable per-callback state behind ONE lock: SCKit calls
        /// `did_output_sample_buffer` with `&self` from its own delivery
        /// queue (the sole locker), so this never contends in steady state.
        inner: Mutex<HandlerInner>,
        warned_channels: std::sync::atomic::AtomicBool,
    }

    /// Per-callback scratch + the shared sample pipeline, all preallocated
    /// and reused — the delivery queue is a real-time-ish context, and the
    /// old per-callback `Vec` collects (one per planar buffer, plus the
    /// interleave output) allocated on every 20 ms callback.
    struct HandlerInner {
        /// Per-channel planar scratch; inner Vecs are reused across callbacks.
        planar: Vec<Vec<f32>>,
        /// Interleave / mono-duplicate output scratch.
        interleaved: Vec<f32>,
        pipeline: CapturePipeline,
    }

    impl SCStreamOutputTrait for OutputHandler {
        fn did_output_sample_buffer(
            &self,
            sample_buffer: CMSampleBuffer,
            of_type: SCStreamOutputType,
        ) {
            if of_type != SCStreamOutputType::Audio {
                return;
            }

            // Retrieve the audio buffer list from the sample buffer.
            // RetainedAudioBufferList derefs to AudioBufferList.
            let audio_buf_list = match sample_buffer.get_audio_buffer_list() {
                Ok(b) => b,
                Err(e) => {
                    tracing::warn!(?e, "sckit get_audio_buffer_list failed");
                    return;
                }
            };

            let mut guard = self.inner.lock();
            // Reborrow as &mut so field-disjoint borrows (planar vs pipeline
            // vs interleaved) split cleanly below.
            let inner = &mut *guard;

            // SCKit delivers planar (non-interleaved) f32: one buffer per channel.
            // Collect each buffer into its reusable scratch slot, then interleave
            // before handing off to the source_channels:2 resampler.
            let mut used = 0usize;
            let mut first_channels = 0u32;
            for buf in audio_buf_list.buffers() {
                if inner.planar.len() <= used {
                    inner.planar.push(Vec::new());
                }
                let dst = &mut inner.planar[used];
                dst.clear();
                let bytes = buf.data();
                // Each sample is 4 bytes (f32, host native byte order).
                dst.reserve(bytes.len() / 4);
                dst.extend(
                    bytes
                        .chunks_exact(4)
                        .map(|b| f32::from_ne_bytes([b[0], b[1], b[2], b[3]])),
                );
                if used == 0 {
                    first_channels = buf.number_channels;
                }
                used += 1;
            }

            match used {
                0 => {}
                1 if first_channels >= 2 => {
                    // Already interleaved stereo (or more) in one buffer — push directly.
                    inner.pipeline.push_f32(&inner.planar[0]);
                }
                1 => {
                    // Mono — duplicate to stereo into the interleave scratch.
                    inner.interleaved.clear();
                    inner.interleaved.reserve(inner.planar[0].len() * 2);
                    for s in &inner.planar[0] {
                        inner.interleaved.push(*s);
                        inner.interleaved.push(*s);
                    }
                    inner.pipeline.push_f32(&inner.interleaved);
                }
                n => {
                    if n > 2
                        && !self
                            .warned_channels
                            .swap(true, std::sync::atomic::Ordering::Relaxed)
                    {
                        tracing::warn!(n, "sckit: unexpected buffer count; interleaving anyway");
                    }
                    crate::audio::planar_to_interleaved_into(
                        &inner.planar[..used],
                        &mut inner.interleaved,
                    );
                    inner.pipeline.push_f32(&inner.interleaved);
                }
            }
        }
    }

    // ── Start capture ─────────────────────────────────────────────────────────

    pub(super) fn start() -> Result<AudioStream> {
        // SCKit must be configured and started from a thread that has a run loop
        // (or at least on a thread where the Objective-C machinery is initialised).
        // We use the init_tx / init_rx pattern from MicSource so that setup errors
        // propagate synchronously to the caller.

        let (tx, rx) = channel();
        let (stop_tx, stop_rx) = oneshot::channel::<()>();
        let (init_tx, init_rx) = std::sync::mpsc::sync_channel::<Result<()>>(1);

        std::thread::Builder::new()
            .name("sckit-loopback".into())
            .spawn(move || {
                // ── Build configuration ──────────────────────────────────────
                let cfg = match build_config() {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = init_tx.send(Err(e));
                        return;
                    }
                };

                // ── Obtain display for filter ────────────────────────────────
                let content = match SCShareableContent::get() {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = init_tx.send(Err(Error::Audio(format!(
                            "sckit SCShareableContent::get: {e:?}"
                        ))));
                        return;
                    }
                };

                let mut displays = content.displays();
                if displays.is_empty() {
                    let _ = init_tx.send(Err(Error::Audio("sckit: no display available".into())));
                    return;
                }
                let display = displays.remove(0);
                let filter = SCContentFilter::new().with_display_excluding_windows(&display, &[]);

                // ── Build resampler + chunker ────────────────────────────────
                let resampler = match Resampler::new(ResamplerSpec {
                    source_hz: 48_000,
                    source_channels: 2,
                }) {
                    Ok(r) => r,
                    Err(e) => {
                        let _ = init_tx.send(Err(e));
                        return;
                    }
                };

                // ── Assemble stream ──────────────────────────────────────────
                let handler = OutputHandler {
                    inner: Mutex::new(HandlerInner {
                        planar: Vec::new(),
                        interleaved: Vec::new(),
                        pipeline: CapturePipeline::new(resampler, tx, "sckit"),
                    }),
                    warned_channels: std::sync::atomic::AtomicBool::new(false),
                };

                let mut stream = SCStream::new(&filter, &cfg);
                stream.add_output_handler(handler, SCStreamOutputType::Audio);

                if let Err(e) = stream.start_capture() {
                    let _ = init_tx.send(Err(Error::Audio(format!("sckit start_capture: {e:?}"))));
                    return;
                }

                // ── Build a stop-wait runtime ────────────────────────────────
                let rt = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(r) => r,
                    Err(e) => {
                        let _ = stream.stop_capture();
                        let _ = init_tx
                            .send(Err(Error::Audio(format!("sckit stop-wait runtime: {e}"))));
                        return;
                    }
                };

                // Signal the parent that startup succeeded.
                let _ = init_tx.send(Ok(()));

                // T8 residual: SCKit exposes no stream-error callback, so a
                // mid-session capture stall leaves the frame channel open (soft
                // zombie) until explicit stop — stop_rx below is the only exit.
                // Runtime/block_on intentionally kept (single-channel wait; no
                // err_rx to multiplex).
                // Block until the stop signal or sender drop.
                let _ = rt.block_on(stop_rx);

                // Stopping the stream and letting `stream` drop cleans up the
                // Objective-C objects on the correct thread.
                stream.stop_capture().ok();
                drop(stream);
            })
            .map_err(|e| Error::Audio(format!("sckit thread spawn: {e}")))?;

        // Propagate any init error to the caller, but never block start() indefinitely: a wedged
        // init (TCC screen-recording dialog, SCKit handshake stall) would otherwise park the
        // calling tokio worker forever and leave the controller's slot stuck Pending, which stop()
        // cannot clear. On timeout, signal the capture thread to halt by dropping the stop sender
        // (it wakes the thread's `block_on(stop_rx)` once it reaches that point so it does not
        // linger holding the capture) and surface a timeout error.
        match init_rx.recv_timeout(INIT_TIMEOUT) {
            Ok(Ok(())) => {}
            Ok(Err(e)) => return Err(e),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                drop(stop_tx);
                return Err(Error::Audio(format!(
                    "sckit audio init timed out ({}s)",
                    INIT_TIMEOUT.as_secs()
                )));
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                return Err(Error::Audio(
                    "sckit thread terminated before signalling init".into(),
                ))
            }
        }

        Ok(AudioStream { rx, stop: stop_tx })
    }

    /// Build the `SCStreamConfiguration` for audio-only loopback capture.
    fn build_config() -> Result<SCStreamConfiguration> {
        SCStreamConfiguration::new()
            .set_captures_audio(true)
            .map_err(|e| Error::Audio(format!("sckit set_captures_audio: {e:?}")))?
            .set_sample_rate(48_000)
            .map_err(|e| Error::Audio(format!("sckit set_sample_rate: {e:?}")))?
            .set_channel_count(2)
            .map_err(|e| Error::Audio(format!("sckit set_channel_count: {e:?}")))?
            .set_excludes_current_process_audio(true)
            .map_err(|e| Error::Audio(format!("sckit set_excludes_current_process_audio: {e:?}")))
    }
}
