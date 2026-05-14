use serde::Serialize;
use tauri::{AppHandle, Emitter};
use voxtide_core::session::CoreEvent;

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum WireEvent {
    SessionStarted { session_id: i64, mode: String },
    SessionStopped { session_id: i64, duration_ms: i64 },
    TranscriptLive {
        status: String,
        text: String,
        language: Option<String>,
        chip: Option<String>,
    },
    TranscriptFinal {
        status: String,
        text: String,
        language: Option<String>,
        chip: Option<String>,
        ts_ms: i64,
    },
    ConnectionState {
        state: String,
        attempt: Option<u32>,
        retry_in_ms: Option<u64>,
    },
    Latency { median_ms: u64 },
}

pub fn forward(app: &AppHandle, ev: CoreEvent) {
    let wire = match ev {
        CoreEvent::SessionStarted { session_id, mode } => {
            WireEvent::SessionStarted { session_id, mode }
        }
        CoreEvent::SessionStopped { session_id, duration_ms } => {
            WireEvent::SessionStopped { session_id, duration_ms }
        }
        CoreEvent::TranscriptLive { status, text, language, chip } => {
            WireEvent::TranscriptLive {
                status: status_str(status),
                text,
                language,
                chip: chip.map(|c| c.to_string()),
            }
        }
        CoreEvent::TranscriptFinal { status, text, language, chip, ts_ms } => {
            WireEvent::TranscriptFinal {
                status: status_str(status),
                text,
                language,
                chip: chip.map(|c| c.to_string()),
                ts_ms,
            }
        }
        CoreEvent::ConnectionState { state, attempt, retry_in_ms } => {
            WireEvent::ConnectionState {
                state: state.into(),
                attempt,
                retry_in_ms,
            }
        }
        CoreEvent::Latency { median_ms } => WireEvent::Latency { median_ms },
    };
    let _ = app.emit("voxtide://event", &wire);
}

fn status_str(s: voxtide_core::translation::tokens::TranslationStatus) -> String {
    use voxtide_core::translation::tokens::TranslationStatus as T;
    match s {
        T::Original => "original".into(),
        T::Translation => "translation".into(),
        T::None => "none".into(),
    }
}
