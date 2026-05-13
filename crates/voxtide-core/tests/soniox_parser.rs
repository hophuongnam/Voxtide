use voxtide_core::translation::tokens::{parse_message, ServerMessage, Token, TranslationStatus};

#[test]
fn parses_two_way_token_pair() {
    let json = r#"{
      "tokens": [
        { "text": "Hello", "is_final": true,  "language": "en", "translation_status": "original" },
        { "text": "Xin chào", "is_final": true, "language": "vi",
          "translation_status": "translation", "source_language": "en" }
      ],
      "final_audio_proc_ms": 4800,
      "total_audio_proc_ms": 5250,
      "finished": false
    }"#;
    let m = parse_message(json).unwrap();
    let ServerMessage::Tokens(t) = m else {
        panic!("expected Tokens");
    };
    assert_eq!(t.tokens.len(), 2);
    let a: &Token = &t.tokens[0];
    assert!(a.is_final);
    assert_eq!(a.language.as_deref(), Some("en"));
    assert_eq!(a.translation_status, TranslationStatus::Original);
    let b: &Token = &t.tokens[1];
    assert_eq!(b.translation_status, TranslationStatus::Translation);
    assert_eq!(b.source_language.as_deref(), Some("en"));
    assert_eq!(t.final_audio_proc_ms, Some(4800));
    assert_eq!(t.total_audio_proc_ms, Some(5250));
    assert!(!t.finished);
}

#[test]
fn parses_finished_marker() {
    let m = parse_message(r#"{"finished": true}"#).unwrap();
    assert!(matches!(m, ServerMessage::Finished));
}

#[test]
fn parses_error_payload() {
    let m = parse_message(r#"{"error":{"code":"unauthorized","message":"bad key"}}"#).unwrap();
    let ServerMessage::Error { code, message } = m else {
        panic!("expected Error");
    };
    assert_eq!(code, "unauthorized");
    assert_eq!(message, "bad key");
}

#[test]
fn parses_non_final_with_speaker() {
    let json = r#"{"tokens":[
        {"text":"th","is_final":false,"language":"en","translation_status":"original","speaker":"1"}
    ]}"#;
    let m = parse_message(json).unwrap();
    let ServerMessage::Tokens(t) = m else {
        panic!("expected Tokens");
    };
    assert!(!t.tokens[0].is_final);
    assert_eq!(t.tokens[0].speaker.as_deref(), Some("1"));
}
