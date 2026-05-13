use std::path::PathBuf;
use voxtide_core::audio::{mock::WavSource, AudioSource, CHUNK_SAMPLES};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

#[tokio::test]
async fn wav_source_emits_one_chunk_per_100ms() {
    let path = fixture("hello-en-16k-mono.wav");
    let src = WavSource::open(&path, false).unwrap();
    let mut stream = src.start().unwrap();
    let mut total = 0usize;
    while let Some(frame) = stream.rx.recv().await {
        assert_eq!(frame.samples.len(), CHUNK_SAMPLES);
        total += frame.samples.len();
    }
    // 2.0 s of audio → 20 frames → 32_000 samples.
    assert_eq!(total, 20 * CHUNK_SAMPLES);
}

#[tokio::test]
async fn wav_source_label_includes_filename() {
    let path = fixture("hello-en-16k-mono.wav");
    let src = WavSource::open(&path, false).unwrap();
    assert!(src.label().contains("hello-en-16k-mono"));
}
