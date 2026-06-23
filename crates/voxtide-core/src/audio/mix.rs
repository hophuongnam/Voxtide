//! Blend a secondary (microphone) source onto a primary (system-audio) source.
//!
//! Both already emit the module's normalized frame (16 kHz mono, 1600-sample
//! chunks), so mixing is element-wise i16 addition. The primary's chunk cadence
//! is the clock: one mixed chunk is emitted per primary chunk, with up to one
//! chunk's worth of buffered mic samples summed in. The mic is a pure overlay —
//! its absence, underrun, or mid-session EOF degrades that span to primary-only
//! and never stalls the primary path. This is what makes mic capture *optional*.

use std::collections::VecDeque;

use tokio::sync::oneshot;

use crate::audio::{channel, AudioSource, AudioStream, CHUNK_SAMPLES};
use crate::{Error, Result};

/// Max mic backlog before oldest samples are dropped to resync (≈300 ms at
/// 16 kHz). Only bites under sustained clock drift (mic running ahead of the
/// system clock); dropping the oldest mic audio is the correct resync.
const MIC_CAP_SAMPLES: usize = CHUNK_SAMPLES * 3;

pub struct MixSource {
    primary: Box<dyn AudioSource>,
    secondary: Box<dyn AudioSource>,
    label: String,
}

impl MixSource {
    pub fn new(primary: Box<dyn AudioSource>, secondary: Box<dyn AudioSource>) -> Self {
        let label = format!("{} + {}", primary.label(), secondary.label());
        Self {
            primary,
            secondary,
            label,
        }
    }
}

impl AudioSource for MixSource {
    fn start(&self) -> Result<AudioStream> {
        // Primary first: its failure IS the capture failure — propagate as-is so
        // the shell routes it to the system-audio (capture-permission) banner.
        let primary = self.primary.start()?;
        // Secondary (mic) second: on failure, stop the primary we just started
        // and mark the error so the shell routes it to the *microphone*-permission
        // banner instead. The "microphone:" prefix is load-bearing — StartError::
        // classify keys the mic-permission route off it.
        let secondary = match self.secondary.start() {
            Ok(s) => s,
            Err(e) => {
                drop(primary);
                return Err(mark_mic_err(e));
            }
        };

        let (tx, rx) = channel();
        let (stop_tx, mut stop_rx) = oneshot::channel::<()>();

        let mut primary_rx = primary.rx;
        let mut secondary_rx = secondary.rx;

        tokio::spawn(async move {
            // Held for the task's lifetime; dropped on exit to halt both inner
            // sources (each detects its stop sender closing).
            let _primary_stop = primary.stop;
            let _secondary_stop = secondary.stop;

            let mut mic_buf: VecDeque<i16> = VecDeque::new();
            let mut mic_open = true;

            loop {
                tokio::select! {
                    // Stop first; then keep the mic buffer fed; then emit on the
                    // primary clock. `biased` keeps the order deterministic.
                    biased;
                    _ = &mut stop_rx => break,
                    // Only polled while open — a closed channel returns None
                    // immediately and would otherwise spin.
                    s = secondary_rx.recv(), if mic_open => match s {
                        Some(frame) => push_mic(&mut mic_buf, frame.samples),
                        None => mic_open = false, // mic dropped → primary-only henceforth
                    },
                    p = primary_rx.recv() => match p {
                        Some(mut frame) => {
                            overlay_chunk(&mut frame.samples, &mut mic_buf);
                            if tx.send(frame).await.is_err() {
                                break; // consumer gone
                            }
                        }
                        None => break, // primary EOS ends the mixed stream
                    },
                }
            }
        });

        Ok(AudioStream { rx, stop: stop_tx })
    }

    fn label(&self) -> &str {
        &self.label
    }
}

/// Append mic samples, dropping the oldest beyond [`MIC_CAP_SAMPLES`].
fn push_mic(mic_buf: &mut VecDeque<i16>, samples: Vec<i16>) {
    mic_buf.extend(samples);
    if mic_buf.len() > MIC_CAP_SAMPLES {
        let overflow = mic_buf.len() - MIC_CAP_SAMPLES;
        mic_buf.drain(..overflow);
    }
}

/// Sum buffered mic samples (oldest first) onto a primary chunk in place,
/// saturating to avoid wrap. Consumes at most `primary.len()` mic samples;
/// any remainder stays buffered for the next chunk. Samples past the mic
/// buffer are left primary-only.
fn overlay_chunk(primary: &mut [i16], mic_buf: &mut VecDeque<i16>) {
    for sample in primary.iter_mut() {
        match mic_buf.pop_front() {
            Some(mic) => *sample = sample.saturating_add(mic),
            None => break,
        }
    }
}

fn mark_mic_err(e: Error) -> Error {
    match e {
        Error::Audio(detail) => Error::Audio(format!("microphone: {detail}")),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_sums_then_passes_through_when_mic_shorter() {
        let mut primary = vec![100i16; 1600];
        let mut mic: VecDeque<i16> = vec![5i16; 800].into();
        overlay_chunk(&mut primary, &mut mic);
        assert!(primary[..800].iter().all(|&s| s == 105), "mic span summed");
        assert!(
            primary[800..].iter().all(|&s| s == 100),
            "rest primary-only"
        );
        assert!(mic.is_empty(), "mic buffer fully drained");
    }

    #[test]
    fn overlay_leaves_remainder_buffered_when_mic_longer() {
        let mut primary = vec![0i16; 1600];
        let mut mic: VecDeque<i16> = vec![7i16; 2000].into();
        overlay_chunk(&mut primary, &mut mic);
        assert!(primary.iter().all(|&s| s == 7));
        assert_eq!(mic.len(), 400, "leftover mic kept for next chunk");
    }

    #[test]
    fn overlay_saturates_instead_of_wrapping() {
        let mut primary = vec![i16::MAX; 4];
        let mut mic: VecDeque<i16> = vec![100i16; 4].into();
        overlay_chunk(&mut primary, &mut mic);
        assert!(primary.iter().all(|&s| s == i16::MAX), "clamped, no wrap");
    }

    #[test]
    fn overlay_noop_when_mic_empty() {
        let mut primary = vec![42i16; 1600];
        let mut mic: VecDeque<i16> = VecDeque::new();
        overlay_chunk(&mut primary, &mut mic);
        assert!(primary.iter().all(|&s| s == 42));
    }

    #[test]
    fn push_mic_caps_backlog_and_keeps_newest() {
        let mut mic: VecDeque<i16> = VecDeque::new();
        push_mic(&mut mic, vec![1i16; MIC_CAP_SAMPLES]);
        push_mic(&mut mic, vec![2i16; 1200]); // 1200 over the cap
        assert_eq!(mic.len(), MIC_CAP_SAMPLES, "backlog bounded");
        assert_eq!(*mic.back().unwrap(), 2, "newest retained");
        assert_eq!(
            *mic.front().unwrap(),
            1,
            "oldest 1200 dropped, older 1s remain"
        );
    }

    #[test]
    fn mark_mic_err_prefixes_audio_only() {
        match mark_mic_err(Error::Audio("denied".into())) {
            Error::Audio(d) => assert_eq!(d, "microphone: denied"),
            _ => panic!("expected Audio"),
        }
    }
}
