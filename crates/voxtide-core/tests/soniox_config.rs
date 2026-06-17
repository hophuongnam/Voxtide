use serde_json::json;
use voxtide_core::translation::soniox::build_initial_config;
use voxtide_core::translation::{Mode, SessionConfig};

#[test]
fn meeting_one_way_translates_source_a_into_target_b() {
    let cfg = SessionConfig {
        api_key: "sk_test".into(),
        mode: Mode::Meeting,
        language_a: "en".into(),
        language_b: "vi".into(),
    };
    let v = build_initial_config(&cfg);
    assert_eq!(v["api_key"], "sk_test");
    assert_eq!(v["model"], "stt-rt-v5");
    assert_eq!(v["audio_format"], "pcm_s16le");
    assert_eq!(v["sample_rate"], 16000);
    assert_eq!(v["num_channels"], 1);
    assert_eq!(v["enable_speaker_diarization"], true);
    assert_eq!(v["enable_endpoint_detection"], true);
    // Source (spoken) language is language_a; it is the only hint.
    assert_eq!(v["language_hints"], json!(["en"]));
    // Target (translation output) language is language_b.
    assert_eq!(
        v["translation"],
        json!({
            "type": "one_way",
            "target_language": "vi"
        })
    );
}

/// Regression: the screenshot scenario — listening to Vietnamese audio,
/// want an English translation. language_a=vi is spoken (hint),
/// language_b=en is the translation target. There is no `mine` flip.
#[test]
fn meeting_one_way_vi_source_into_en_target() {
    let cfg = SessionConfig {
        api_key: "sk_test".into(),
        mode: Mode::Meeting,
        language_a: "vi".into(),
        language_b: "en".into(),
    };
    let v = build_initial_config(&cfg);
    assert_eq!(v["language_hints"], json!(["vi"]));
    assert_eq!(
        v["translation"],
        json!({
            "type": "one_way",
            "target_language": "en"
        })
    );
}

#[test]
fn conversation_two_way_config_emits_both_languages() {
    let cfg = SessionConfig {
        api_key: "sk_test".into(),
        mode: Mode::Conversation,
        language_a: "en".into(),
        language_b: "ja".into(),
    };
    let v = build_initial_config(&cfg);
    assert_eq!(
        v["translation"],
        json!({
            "type": "two_way",
            "language_a": "en",
            "language_b": "ja"
        })
    );
    assert_eq!(v["language_hints"], serde_json::Value::Null);
}
