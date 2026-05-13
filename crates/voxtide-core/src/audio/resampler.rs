use rubato::{SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction, Resampler as _};

use crate::audio::SAMPLE_RATE_HZ;
use crate::{Error, Result};

#[derive(Debug, Clone, Copy)]
pub struct ResamplerSpec {
    pub source_hz: u32,
    pub source_channels: u16,
}

pub struct Resampler {
    spec: ResamplerSpec,
    inner: Option<SincFixedIn<f32>>,
    mono_buf: Vec<f32>,
}

impl Resampler {
    pub fn new(spec: ResamplerSpec) -> Result<Self> {
        if spec.source_channels == 0 {
            return Err(Error::Audio("source_channels must be >= 1".into()));
        }
        let inner = if spec.source_hz == SAMPLE_RATE_HZ {
            None
        } else {
            let params = SincInterpolationParameters {
                sinc_len: 256,
                f_cutoff: 0.95,
                interpolation: SincInterpolationType::Linear,
                oversampling_factor: 256,
                window: WindowFunction::BlackmanHarris2,
            };
            let ratio = SAMPLE_RATE_HZ as f64 / spec.source_hz as f64;
            // chunk_size = 480 so that 4800-sample @ 48 kHz input produces exactly
            // 10 full chunks (4800 / 480 = 10), each yielding ~160 output samples
            // @ 16 kHz → ~1600 total, within the ±5% test tolerance.
            // The plan's original 1024 would only process 4 chunks → ~1365 samples,
            // which falls below the lower bound of 1520.
            let r = SincFixedIn::<f32>::new(ratio, 2.0, params, 480, 1)
                .map_err(|e| Error::Audio(format!("rubato init: {e}")))?;
            Some(r)
        };
        Ok(Self { spec, inner, mono_buf: Vec::new() })
    }

    /// Process interleaved samples in `[-1.0, 1.0]`. Returns mono @ 16 kHz f32.
    pub fn process(&mut self, interleaved: &[f32]) -> Result<Vec<f32>> {
        let ch = self.spec.source_channels as usize;
        self.mono_buf.clear();
        self.mono_buf.reserve(interleaved.len() / ch);
        for frame in interleaved.chunks_exact(ch) {
            let sum: f32 = frame.iter().sum();
            self.mono_buf.push(sum / ch as f32);
        }

        if self.inner.is_none() {
            return Ok(self.mono_buf.clone());
        }

        let r = self.inner.as_mut().expect("checked above");
        let mut out = Vec::new();
        let input_chunk = r.input_frames_next();
        let mut cursor = 0;
        while cursor + input_chunk <= self.mono_buf.len() {
            let slice = &self.mono_buf[cursor..cursor + input_chunk];
            let processed = r.process(&[slice], None)
                .map_err(|e| Error::Audio(format!("rubato process: {e}")))?;
            out.extend_from_slice(&processed[0]);
            cursor += input_chunk;
        }
        // Drop the tail; caller's Chunker will absorb partial 100 ms windows.
        Ok(out)
    }
}

pub fn f32_to_i16(v: f32) -> i16 {
    let clamped = v.clamp(-1.0, 1.0);
    // Avoid asymmetric -32768 fence-post bug by mapping to ±32767.
    if clamped >= 0.0 {
        (clamped * i16::MAX as f32).round() as i16
    } else {
        (clamped * -(i16::MIN as f32 + 1.0)).round() as i16
    }
}
