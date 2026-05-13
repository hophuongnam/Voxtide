use serde_json::{json, Value};

use crate::translation::{Mode, SessionConfig, WhichLang};

pub const SONIOX_WS: &str = "wss://stt-rt.soniox.com/transcribe-websocket";
pub const MODEL: &str = "stt-rt-v4";

pub fn build_initial_config(cfg: &SessionConfig) -> Value {
    let mut base = json!({
        "api_key": cfg.api_key,
        "model": MODEL,
        "audio_format": "pcm_s16le",
        "sample_rate": 16000,
        "num_channels": 1,
        "enable_endpoint_detection": true,
        "enable_speaker_diarization": true,
    });

    let (mine_lang, other_lang) = match cfg.mine {
        WhichLang::A => (cfg.language_a.clone(), cfg.language_b.clone()),
        WhichLang::B => (cfg.language_b.clone(), cfg.language_a.clone()),
    };

    match cfg.mode {
        Mode::Meeting => {
            base["language_hints"] = json!([other_lang]);
            base["translation"] = json!({
                "type": "one_way",
                "target_language": mine_lang,
            });
        }
        Mode::Conversation => {
            base["translation"] = json!({
                "type": "two_way",
                "language_a": cfg.language_a,
                "language_b": cfg.language_b,
            });
        }
    }
    base
}
