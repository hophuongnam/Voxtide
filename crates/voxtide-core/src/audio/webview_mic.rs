//! Push-fed audio source for Android Path B: the WebView captures the mic via
//! getUserMedia and streams PCM to Rust through the `feed_mic_pcm` command,
//! which pushes into the live [`MicFeed`] sink that this source drains.

use std::sync::{Arc, Mutex};

use tokio::sync::{mpsc, oneshot};

use crate::audio::resampler::f32_to_i16;
use crate::audio::{channel, AudioSource, AudioStream, Chunker};
use crate::Result;

/// Shared sink the `feed_mic_pcm` command writes raw mic PCM into. `None` when
/// no Android capture session is active. Cloned into both `AppState` (for the
/// command) and the active [`WebViewMicSource`].
pub type MicFeed = Arc<Mutex<Option<mpsc::Sender<Vec<f32>>>>>;

pub fn new_mic_feed() -> MicFeed {
    Arc::new(Mutex::new(None))
}

/// `AudioSource` fed by PCM pushed from the WebView. The JS side forces a
/// 16 kHz `AudioContext`, so the pushed samples are already mono f32 at 16 kHz;
/// this only converts f32->i16 and chunks into 100 ms `AudioFrame`s.
/// ponytail: assumes 16 kHz input (JS forces it). If a device can't honor a
/// 16 kHz AudioContext, resample in the worklet or add a Resampler here — add when needed.
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
        let (pcm_tx, mut pcm_rx) = mpsc::channel::<Vec<f32>>(64);

        // Register the live sink so feed_mic_pcm pushes into THIS session.
        *self.feed.lock().unwrap() = Some(pcm_tx);

        tokio::spawn(async move {
            let mut chunker = Chunker::new();
            loop {
                tokio::select! {
                    biased;
                    _ = &mut stop_rx => break,
                    maybe = pcm_rx.recv() => match maybe {
                        Some(pcm) => {
                            let i16s: Vec<i16> = pcm.into_iter().map(f32_to_i16).collect();
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
    use crate::audio::CHUNK_SAMPLES;

    #[tokio::test]
    async fn pushed_pcm_becomes_i16_frames() {
        let feed = new_mic_feed();
        let src = WebViewMicSource::new(feed.clone());
        let mut stream = src.start().unwrap();

        // start() registers the live sink; grab it and push > one chunk of f32.
        let tx = feed.lock().unwrap().clone().expect("sink registered");
        let samples = vec![1.0f32; CHUNK_SAMPLES + 10]; // full-scale -> i16::MAX
        tx.send(samples).await.unwrap();

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
            sink.send(vec![0.0f32]).await.is_err(),
            "drain task should have exited and closed its receiver"
        );
    }
}
