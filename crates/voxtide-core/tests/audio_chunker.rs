use voxtide_core::audio::{AudioFrame, Chunker, CHUNK_SAMPLES};

#[test]
fn chunk_samples_matches_100ms_at_16k() {
    assert_eq!(CHUNK_SAMPLES, 1600);
}

#[test]
fn chunker_emits_full_chunks_only() {
    let mut chunker = Chunker::new();
    let out: Vec<AudioFrame> = chunker.push(&vec![0i16; 800]).collect();
    assert!(out.is_empty(), "half a chunk should buffer, not emit");
    let out: Vec<AudioFrame> = chunker.push(&vec![1i16; 800]).collect();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].samples.len(), CHUNK_SAMPLES);
    assert_eq!(out[0].samples[0], 0);
    assert_eq!(out[0].samples[CHUNK_SAMPLES - 1], 1);
}

#[test]
fn chunker_emits_multiple_chunks_when_overfed() {
    let mut chunker = Chunker::new();
    let out: Vec<AudioFrame> = chunker.push(&vec![0i16; 3 * CHUNK_SAMPLES + 200]).collect();
    assert_eq!(out.len(), 3);
    for f in &out {
        assert_eq!(f.samples.len(), CHUNK_SAMPLES);
    }
}

#[test]
fn audio_frame_to_le_bytes_doubles_sample_count() {
    let f = AudioFrame {
        samples: vec![0i16, 1, -1, 256],
    };
    let bytes = f.to_le_bytes();
    assert_eq!(bytes.len(), 8);
    assert_eq!(&bytes[0..2], &[0, 0]);
    assert_eq!(&bytes[2..4], &[1, 0]);
    assert_eq!(&bytes[4..6], &[0xff, 0xff]);
    assert_eq!(&bytes[6..8], &[0, 1]);
}
