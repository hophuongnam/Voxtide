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
        capture_mic: false,
    };
    let v = build_initial_config(&cfg, "");
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
        capture_mic: false,
    };
    let v = build_initial_config(&cfg, "");
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
        capture_mic: false,
    };
    let v = build_initial_config(&cfg, "");
    assert_eq!(
        v["translation"],
        json!({
            "type": "two_way",
            "language_a": "en",
            "language_b": "ja"
        })
    );
    // Both languages are hinted for recognition in two-way mode (either side
    // can be spoken into the stream).
    assert_eq!(v["language_hints"], json!(["en", "ja"]));
}

/// Meeting mode with the local mic blended in goes two-way: the remote side
/// (language_a, via system audio) and the local speaker (language_b, via mic)
/// are both transcribed and cross-translated. Without this, the mic's
/// language_b speech would be hinted for language_a and never translated.
#[test]
fn meeting_with_capture_mic_is_two_way() {
    let cfg = SessionConfig {
        api_key: "sk_test".into(),
        mode: Mode::Meeting,
        language_a: "en".into(),
        language_b: "vi".into(),
        capture_mic: true,
    };
    let v = build_initial_config(&cfg, "");
    assert_eq!(
        v["translation"],
        json!({
            "type": "two_way",
            "language_a": "en",
            "language_b": "vi"
        })
    );
    // Both languages hinted: the remote (a) via system audio AND the local
    // mic (b) can each be spoken into the blended stream.
    assert_eq!(v["language_hints"], json!(["en", "vi"]));
}

/// Context: a non-empty user string is sent as Soniox's `context.text`
/// (free-text section); blank context omits the key entirely so the wire
/// config is unchanged for users who never set it.
#[test]
fn context_text_included_only_when_non_empty() {
    let cfg = SessionConfig {
        api_key: "sk_test".into(),
        mode: Mode::Conversation,
        language_a: "en".into(),
        language_b: "ja".into(),
        capture_mic: false,
    };
    let with_ctx = build_initial_config(&cfg, "  Acme Corp; speakers: Nam, Yuki  ");
    assert_eq!(
        with_ctx["context"],
        json!({ "text": "Acme Corp; speakers: Nam, Yuki" }),
        "context.text should carry the trimmed user string"
    );

    let blank = build_initial_config(&cfg, "   ");
    assert_eq!(
        blank["context"],
        serde_json::Value::Null,
        "blank/whitespace context must omit the key"
    );
}
