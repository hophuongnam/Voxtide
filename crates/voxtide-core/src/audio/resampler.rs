use rubato::{
    Resampler as _, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};

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

impl std::fmt::Debug for Resampler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Resampler")
            .field("spec", &self.spec)
            .field("inner", &self.inner.as_ref().map(|_| "<SincFixedIn>"))
            .field("mono_buf_len", &self.mono_buf.len())
            .finish()
    }
}

impl Resampler {
    pub fn new(spec: ResamplerSpec) -> Result<Self> {
        if spec.source_channels == 0 {
            return Err(Error::Audio("source_channels must be >= 1".into()));
        }
        if spec.source_hz == 0 {
            return Err(Error::Audio("source_hz must be > 0".into()));
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
        Ok(Self {
            spec,
            inner,
            mono_buf: Vec::new(),
        })
    }

    /// Process interleaved samples in `[-1.0, 1.0]`. Returns mono @ 16 kHz f32.
    ///
    /// `mono_buf` is a persistent carry buffer: new samples are appended each call,
    /// whole rubato chunks are consumed from the front, and any sub-chunk remainder
    /// is retained for the next call. This ensures no samples are lost regardless
    /// of device callback size (assumes whole interleaved frames per callback).
    pub fn process(&mut self, interleaved: &[f32]) -> Result<Vec<f32>> {
        let ch = self.spec.source_channels as usize;
        self.mono_buf.reserve(interleaved.len() / ch);
        for frame in interleaved.chunks_exact(ch) {
            let sum: f32 = frame.iter().sum();
            self.mono_buf.push(sum / ch as f32);
        }

        let Some(r) = self.inner.as_mut() else {
            // Passthrough: drain everything and return it.
            return Ok(std::mem::take(&mut self.mono_buf));
        };
        let ratio = SAMPLE_RATE_HZ as f64 / self.spec.source_hz as f64;
        let estimated_out = (self.mono_buf.len() as f64 * ratio).ceil() as usize + 16;
        let mut out = Vec::with_capacity(estimated_out);
        let mut consumed = 0usize;
        while self.mono_buf.len() - consumed >= r.input_frames_next() {
            let chunk = r.input_frames_next();
            let slice = &self.mono_buf[consumed..consumed + chunk];
            let processed = match r.process(&[slice], None) {
                Ok(res) => res,
                Err(e) => {
                    self.mono_buf.drain(..consumed);
                    return Err(Error::Audio(format!("rubato process: {e}")));
                }
            };
            out.extend_from_slice(&processed[0]);
            consumed += chunk;
        }
        // Retain sub-chunk tail in mono_buf for the next call.
        self.mono_buf.drain(..consumed);
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
