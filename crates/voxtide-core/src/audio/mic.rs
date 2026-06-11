use cpal::traits::{DeviceTrait, HostTrait};

use crate::audio::cpal_pipeline::{start_capture, CpalCaptureSpec};
use crate::audio::{AudioSource, AudioStream};
use crate::{Error, Result};

// ─── Device listing ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MicDevice {
    pub id: String,
    pub label: String,
    pub default: bool,
}

pub fn list_input_devices() -> Result<Vec<MicDevice>> {
    let host = cpal::default_host();
    let default = host.default_input_device().and_then(|d| d.name().ok());
    let devices = host
        .input_devices()
        .map_err(|e| Error::Audio(format!("cpal input_devices: {e}")))?;
    let mut out = Vec::new();
    for d in devices {
        let name = d
            .name()
            .map_err(|e| Error::Audio(format!("cpal name: {e}")))?;
        out.push(MicDevice {
            id: name.clone(),
            default: default.as_deref() == Some(&name),
            label: name,
        });
    }
    Ok(out)
}

// ─── MicSource ───────────────────────────────────────────────────────────────

pub struct MicSource {
    device_id: Option<String>,
    label: String,
}

impl MicSource {
    pub fn default_device() -> Result<Self> {
        Ok(Self {
            device_id: None,
            label: "Default microphone".into(),
        })
    }

    pub fn by_id(id: &str) -> Self {
        Self {
            device_id: Some(id.to_string()),
            label: id.to_string(),
        }
    }
}

fn pick_device(host: &cpal::Host, want_id: Option<&str>) -> Result<cpal::Device> {
    if let Some(id) = want_id {
        for d in host
            .input_devices()
            .map_err(|e| Error::Audio(e.to_string()))?
        {
            if d.name().ok().as_deref() == Some(id) {
                return Ok(d);
            }
        }
        return Err(Error::Audio(format!("mic device not found: {id}")));
    }
    host.default_input_device()
        .ok_or_else(|| Error::Audio("no default input device".into()))
}

impl AudioSource for MicSource {
    fn start(&self) -> Result<AudioStream> {
        let device_id = self.device_id.clone();
        start_capture(CpalCaptureSpec {
            thread_name: format!("mic-{}", self.label),
            label: "mic",
            // A mic always fires callbacks (real silence is still data);
            // injection would corrupt timing.
            silence_keepalive: false,
            open: Box::new(move |host| {
                let device = pick_device(host, device_id.as_deref())?;
                let config = device
                    .default_input_config()
                    .map_err(|e| Error::Audio(format!("cpal default_input_config: {e}")))?;
                Ok((device, config))
            }),
        })
    }

    fn label(&self) -> &str {
        &self.label
    }
}
