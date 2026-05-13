use serde_json::json;
use voxtide_core::translation::soniox::build_initial_config;
use voxtide_core::translation::{Mode, SessionConfig, WhichLang};

#[test]
fn meeting_one_way_config_targets_my_language() {
    let cfg = SessionConfig {
        api_key: "sk_test".into(),
        mode: Mode::Meeting,
        language_a: "en".into(),
        language_b: "vi".into(),
        mine: WhichLang::B,
    };
    let v = build_initial_config(&cfg);
    assert_eq!(v["api_key"], "sk_test");
    assert_eq!(v["model"], "stt-rt-v4");
    assert_eq!(v["audio_format"], "pcm_s16le");
    assert_eq!(v["sample_rate"], 16000);
    assert_eq!(v["num_channels"], 1);
    assert_eq!(v["enable_speaker_diarization"], true);
    assert_eq!(v["enable_endpoint_detection"], true);
    assert_eq!(v["language_hints"], json!(["en"]));
    assert_eq!(v["translation"], json!({
        "type": "one_way",
        "target_language": "vi"
    }));
}

#[test]
fn conversation_two_way_config_emits_both_languages() {
    let cfg = SessionConfig {
        api_key: "sk_test".into(),
        mode: Mode::Conversation,
        language_a: "en".into(),
        language_b: "ja".into(),
        mine: WhichLang::A,
    };
    let v = build_initial_config(&cfg);
    assert_eq!(v["translation"], json!({
        "type": "two_way",
        "language_a": "en",
        "language_b": "ja"
    }));
    assert_eq!(v["language_hints"], serde_json::Value::Null);
}
