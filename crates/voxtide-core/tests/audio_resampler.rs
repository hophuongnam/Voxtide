use voxtide_core::audio::resampler::{Resampler, ResamplerSpec};
use voxtide_core::audio::SAMPLE_RATE_HZ;

#[test]
fn passthrough_when_source_rate_matches_target() {
    let mut r = Resampler::new(ResamplerSpec { source_hz: SAMPLE_RATE_HZ, source_channels: 1 }).unwrap();
    let input: Vec<f32> = (0..1600).map(|i| (i as f32) / 1600.0).collect();
    let out = r.process(&input).unwrap();
    assert_eq!(out.len(), 1600);
    assert!((out[0] - input[0]).abs() < 1e-6);
    assert!((out[1599] - input[1599]).abs() < 1e-6);
}

#[test]
fn downsamples_48k_to_16k_within_5_percent_length() {
    let mut r = Resampler::new(ResamplerSpec { source_hz: 48_000, source_channels: 1 }).unwrap();
    let input = vec![0.0f32; 4800]; // 100 ms @ 48 kHz
    let out = r.process(&input).unwrap();
    // Expect ~1600 samples (100 ms @ 16 kHz). Allow ±5%.
    let lo = (1600.0 * 0.95) as usize;
    let hi = (1600.0 * 1.05) as usize;
    assert!((lo..=hi).contains(&out.len()), "got {} samples", out.len());
}

#[test]
fn stereo_downmix_to_mono_averages_channels() {
    let mut r = Resampler::new(ResamplerSpec { source_hz: 16_000, source_channels: 2 }).unwrap();
    // Interleaved stereo: [L=1.0, R=-1.0, L=0.5, R=0.5, ...]
    let input = vec![1.0_f32, -1.0, 0.5, 0.5, 0.2, 0.4];
    let out = r.process(&input).unwrap();
    assert_eq!(out.len(), 3);
    assert!((out[0] - 0.0).abs() < 1e-6);
    assert!((out[1] - 0.5).abs() < 1e-6);
    assert!((out[2] - 0.3).abs() < 1e-6);
}

#[test]
fn new_rejects_zero_source_hz() {
    let err = Resampler::new(ResamplerSpec { source_hz: 0, source_channels: 1 }).unwrap_err();
    assert!(err.to_string().contains("source_hz"));
}

#[test]
fn f32_to_i16_clips() {
    use voxtide_core::audio::resampler::f32_to_i16;
    assert_eq!(f32_to_i16(0.0), 0);
    assert_eq!(f32_to_i16(1.0), i16::MAX);
    assert_eq!(f32_to_i16(-1.0), i16::MIN + 1);
    assert_eq!(f32_to_i16(2.0), i16::MAX);
    assert_eq!(f32_to_i16(-2.0), i16::MIN + 1);
}
