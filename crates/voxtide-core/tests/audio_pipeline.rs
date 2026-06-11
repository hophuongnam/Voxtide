//! Behavioral contract of the shared capture pipeline: any cpal-shaped input
//! (arbitrary sample type, callback-sized buffers) comes out as 16 kHz mono
//! s16le 1600-sample AudioFrames on the channel, with no samples lost to
//! callback boundaries.

use tokio::sync::mpsc;
use voxtide_core::audio::cpal_pipeline::CapturePipeline;
use voxtide_core::audio::resampler::{Resampler, ResamplerSpec};
use voxtide_core::audio::{AudioFrame, CHUNK_SAMPLES};

fn drain(rx: &mut mpsc::Receiver<AudioFrame>) -> Vec<AudioFrame> {
    let mut out = Vec::new();
    while let Ok(f) = rx.try_recv() {
        out.push(f);
    }
    out
}

#[test]
fn push_converted_produces_full_chunks_across_callbacks() {
    // 48 kHz stereo i16 input in 512-frame callbacks → 16 kHz mono frames.
    let (tx, mut rx) = voxtide_core::audio::channel();
    let resampler = Resampler::new(ResamplerSpec {
        source_hz: 48_000,
        source_channels: 2,
    })
    .expect("resampler");
    let mut pipe = CapturePipeline::new(resampler, tx, "test");
    let callback = vec![1000i16; 512 * 2];
    // 300 callbacks × 512 frames = 3.2 s @ 48 kHz → ~51200 mono samples @ 16 kHz
    // → 32 full 1600-sample frames (minus at most one carry chunk).
    for _ in 0..300 {
        pipe.push_converted(&callback, |s| s as f32 / i16::MAX as f32);
    }
    let frames = drain(&mut rx);
    assert!(
        frames.len() >= 31,
        "expected ≥31 full frames, got {}",
        frames.len()
    );
    assert!(frames.iter().all(|f| f.samples.len() == CHUNK_SAMPLES));
    // Non-silent input must produce non-silent output.
    assert!(frames.iter().any(|f| f.samples.iter().any(|s| *s != 0)));
}

#[test]
fn push_f32_passthrough_emits_one_frame_per_chunk() {
    // Source already 16 kHz mono → passthrough; 100 ms of input per call.
    let (tx, mut rx) = voxtide_core::audio::channel();
    let resampler = Resampler::new(ResamplerSpec {
        source_hz: 16_000,
        source_channels: 1,
    })
    .expect("resampler");
    let mut pipe = CapturePipeline::new(resampler, tx, "test");
    let chunk = vec![0.5f32; CHUNK_SAMPLES];
    for _ in 0..5 {
        pipe.push_f32(&chunk);
    }
    let frames = drain(&mut rx);
    assert_eq!(frames.len(), 5);
    assert!(frames.iter().all(|f| f.samples.len() == CHUNK_SAMPLES));
}

#[test]
fn sub_chunk_callbacks_carry_remainders() {
    // 441-frame stereo callbacks @ 44.1 kHz (the classic non-multiple case the
    // resampler carry-fix exists for) must still flow through the pipeline.
    let (tx, mut rx) = voxtide_core::audio::channel();
    let resampler = Resampler::new(ResamplerSpec {
        source_hz: 44_100,
        source_channels: 2,
    })
    .expect("resampler");
    let mut pipe = CapturePipeline::new(resampler, tx, "test");
    let callback = vec![0.25f32; 441 * 2];
    // 100 callbacks = 1.0 s @ 44.1 kHz → ~16000 samples @ 16 kHz → ≥9 full frames.
    for _ in 0..100 {
        pipe.push_f32(&callback);
    }
    let frames = drain(&mut rx);
    assert!(
        frames.len() >= 9,
        "expected ≥9 frames from 1s of audio, got {}",
        frames.len()
    );
}
