#![cfg(target_os = "windows")]
//! Windows system loopback audio capture via WASAPI render endpoint.
//!
//! cpal 0.15 exposes WASAPI loopback by calling `build_input_stream` on a
//! device obtained from `host.default_output_device()` (or any output device).
//! Windows internally routes this as a WASAPI loopback capture stream.

use cpal::traits::{DeviceTrait, HostTrait};

use crate::audio::cpal_pipeline::{start_capture, CpalCaptureSpec};
use crate::audio::{AudioSource, AudioStream};
use crate::{Error, Result};

// ─── Device listing ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoopbackSource {
    pub id: String,
    pub label: String,
    pub default: bool,
}

/// List all available render (output) endpoints that can be used as loopback sources.
pub fn list_loopback_sources() -> Result<Vec<LoopbackSource>> {
    let host = cpal::default_host();
    let default_name = host.default_output_device().and_then(|d| d.name().ok());
    let mut out = Vec::new();
    for d in host
        .output_devices()
        .map_err(|e| Error::Audio(format!("cpal output_devices: {e}")))?
    {
        let name = d
            .name()
            .map_err(|e| Error::Audio(format!("cpal device name: {e}")))?;
        out.push(LoopbackSource {
            default: default_name.as_deref() == Some(&name),
            label: format!("{name} (loopback)"),
            id: name,
        });
    }
    Ok(out)
}

// ─── WinLoopbackSource ────────────────────────────────────────────────────────

pub struct WinLoopbackSource {
    device_id: Option<String>,
    label: String,
}

impl WinLoopbackSource {
    /// Capture from the system default render (output) endpoint.
    pub fn default_render() -> Self {
        Self {
            device_id: None,
            label: "Default render (loopback)".into(),
        }
    }

    /// Capture from the render endpoint identified by `id`.
    pub fn by_id(id: &str) -> Self {
        Self {
            device_id: Some(id.to_string()),
            label: format!("{id} (loopback)"),
        }
    }
}

fn pick_render_device(host: &cpal::Host, want_id: Option<&str>) -> Result<cpal::Device> {
    if let Some(id) = want_id {
        for d in host
            .output_devices()
            .map_err(|e| Error::Audio(e.to_string()))?
        {
            if d.name().ok().as_deref() == Some(id) {
                return Ok(d);
            }
        }
        return Err(Error::Audio(format!("render device not found: {id}")));
    }
    host.default_output_device()
        .ok_or_else(|| Error::Audio("no default render device".into()))
}

impl AudioSource for WinLoopbackSource {
    fn start(&self) -> Result<AudioStream> {
        let device_id = self.device_id.clone();
        start_capture(CpalCaptureSpec {
            thread_name: format!("wasapi-loopback-{}", self.label),
            label: "wasapi",
            // WASAPI loopback fires no callbacks while nothing is playing;
            // the keepalive injects silence so Soniox doesn't idle-close.
            silence_keepalive: true,
            open: Box::new(move |host| {
                let device = pick_render_device(host, device_id.as_deref())?;
                // Use the default *output* config — cpal/WASAPI loopback
                // inherits the render endpoint's native format.
                let config = device
                    .default_output_config()
                    .map_err(|e| Error::Audio(format!("cpal default_output_config: {e}")))?;
                Ok((device, config))
            }),
        })
    }

    fn label(&self) -> &str {
        &self.label
    }
}
