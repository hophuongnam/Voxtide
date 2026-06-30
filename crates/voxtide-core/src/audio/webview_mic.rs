//! Push-fed audio source for Android Path B: the WebView captures the mic via
//! getUserMedia and streams PCM to Rust through the `feed_mic_pcm` command,
//! which pushes into the live [`MicFeed`] sink that this source drains.

use std::sync::{Arc, Mutex};

use tokio::sync::{mpsc, oneshot};

use crate::audio::resampler::{f32_to_i16, Resampler, ResamplerSpec};
use crate::audio::{channel, AudioSource, AudioStream, Chunker};
use crate::Result;

/// Shared sink the `feed_mic_pcm` command writes raw mic PCM into. `None` when
/// no Android capture session is active. Cloned into both `AppState` (for the
/// command) and the active [`WebViewMicSource`].
pub type MicFeed = Arc<Mutex<Option<mpsc::Sender<WebViewPcmBatch>>>>;

#[derive(Debug, Clone)]
pub struct WebViewPcmBatch {
    pub samples: Vec<f32>,
    pub sample_rate_hz: u32,
}

pub fn new_mic_feed() -> MicFeed {
    Arc::new(Mutex::new(None))
}

/// `AudioSource` fed by PCM pushed from the WebView. JS requests a 16 kHz
/// `AudioContext`, but Android WebView may return a hardware-native rate. Each
/// pushed batch includes the actual rate, so this source normalizes to Voxtide's
/// 16 kHz mono contract before chunking.
pub struct WebViewMicSource {
    feed: MicFeed,
}

impl WebViewMicSource {
    pub fn new(feed: MicFeed) -> Self {
        Self { feed }
    }
}

impl AudioSource for WebViewMicSource {
    fn start(&self) -> Result<AudioStream> {
        let (frame_tx, frame_rx) = channel(); // mpsc::Sender/Receiver<AudioFrame>
        let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
        let (pcm_tx, mut pcm_rx) = mpsc::channel::<WebViewPcmBatch>(64);

        // Register the live sink so feed_mic_pcm pushes into THIS session.
        *self.feed.lock().unwrap() = Some(pcm_tx);

        tokio::spawn(async move {
            let mut chunker = Chunker::new();
            let mut resampler: Option<(u32, Resampler)> = None;
            let mut resampled = Vec::new();
            loop {
                tokio::select! {
                    biased;
                    _ = &mut stop_rx => break,
                    maybe = pcm_rx.recv() => match maybe {
                        Some(batch) => {
                            if batch.sample_rate_hz == 0 {
                                tracing::warn!("dropping WebView mic batch with zero sample rate");
                                continue;
                            }
                            if resampler.as_ref().map(|(hz, _)| *hz) != Some(batch.sample_rate_hz) {
                                match Resampler::new(ResamplerSpec {
                                    source_hz: batch.sample_rate_hz,
                                    source_channels: 1,
                                }) {
                                    Ok(r) => resampler = Some((batch.sample_rate_hz, r)),
                                    Err(e) => {
                                        tracing::warn!(error = %e, sample_rate_hz = batch.sample_rate_hz, "dropping WebView mic batch; resampler init failed");
                                        continue;
                                    }
                                }
                            }
                            let Some((_, r)) = resampler.as_mut() else { continue };
                            if let Err(e) = r.process_into(&batch.samples, &mut resampled) {
                                tracing::warn!(error = %e, sample_rate_hz = batch.sample_rate_hz, "dropping WebView mic batch; resample failed");
                                continue;
                            }
                            let i16s: Vec<i16> = resampled.iter().copied().map(f32_to_i16).collect();
                            for frame in chunker.push(&i16s) {
                                if frame_tx.send(frame).await.is_err() {
                                    // consumer (session worker) gone
                                    return;
                                }
                            }
                        }
                        None => break,
                    },
                }
            }
        });

        Ok(AudioStream {
            rx: frame_rx,
            stop: stop_tx,
        })
    }

    fn label(&self) -> &str {
        "WebView microphone"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::{CHUNK_SAMPLES, SAMPLE_RATE_HZ};

    #[tokio::test]
    async fn pushed_pcm_becomes_i16_frames() {
        let feed = new_mic_feed();
        let src = WebViewMicSource::new(feed.clone());
        let mut stream = src.start().unwrap();

        // start() registers the live sink; grab it and push > one chunk of f32.
        let tx = feed.lock().unwrap().clone().expect("sink registered");
        tx.send(WebViewPcmBatch {
            samples: vec![1.0f32; CHUNK_SAMPLES + 10], // full-scale -> i16::MAX
            sample_rate_hz: SAMPLE_RATE_HZ,
        })
        .await
        .unwrap();

        let frame = stream.rx.recv().await.expect("a frame");
        assert_eq!(frame.samples.len(), CHUNK_SAMPLES);
        assert!(frame.samples.iter().all(|&s| s == i16::MAX));
    }

    #[tokio::test]
    async fn stop_ends_drain_task_without_clearing_slot() {
        let feed = new_mic_feed();
        let stream = WebViewMicSource::new(feed.clone()).start().unwrap();
        let sink = feed.lock().unwrap().clone().expect("sink registered");
        drop(stream.stop); // signal the drain task to exit
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        // Drain task exited: its pcm_rx dropped, so the registered sender is now closed.
        // The slot is intentionally NOT cleared (a dead sender is harmless; next start() overwrites it).
        assert!(
            sink.send(WebViewPcmBatch {
                samples: vec![0.0f32],
                sample_rate_hz: SAMPLE_RATE_HZ,
            })
            .await
            .is_err(),
            "drain task should have exited and closed its receiver"
        );
    }

    #[tokio::test]
    async fn resamples_webview_native_rate_to_standard_frames() {
        let feed = new_mic_feed();
        let src = WebViewMicSource::new(feed.clone());
        let mut stream = src.start().unwrap();
        let tx = feed.lock().unwrap().clone().expect("sink registered");

        tx.send(WebViewPcmBatch {
            samples: vec![0.5f32; 48_000],
            sample_rate_hz: 48_000,
        })
        .await
        .unwrap();

        let frame = tokio::time::timeout(std::time::Duration::from_secs(1), stream.rx.recv())
            .await
            .expect("resampled frame timeout")
            .expect("a resampled frame");
        assert_eq!(frame.samples.len(), CHUNK_SAMPLES);
        assert!(frame.samples.iter().any(|&s| s != 0));
    }
}
