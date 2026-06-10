use voxtide_core::translation::mock::MockProvider;
use voxtide_core::translation::tokens::TranslationStatus;
use voxtide_core::translation::{
    FinalToken, Mode, SessionConfig, TranslationEvent, TranslationProvider,
};

#[tokio::test]
async fn mock_replays_a_script_after_open() {
    let script = vec![
        TranslationEvent::Connected,
        TranslationEvent::Live {
            text: "Hel".into(),
            language: Some("en".into()),
            status: TranslationStatus::Original,
            speaker: Some("1".into()),
        },
        TranslationEvent::Finals {
            tokens: vec![FinalToken {
                text: "Hello".into(),
                language: Some("en".into()),
                status: TranslationStatus::Original,
                speaker: Some("1".into()),
                ts_ms: 1,
            }],
            lag_ms: None,
        },
        TranslationEvent::Stopped,
    ];
    let mut p = MockProvider::with_script(script);
    p.open(SessionConfig {
        api_key: "".into(),
        mode: Mode::Conversation,
        language_a: "en".into(),
        language_b: "vi".into(),
    })
    .await
    .unwrap();

    let mut count = 0;
    while let Some(ev) = p.next_event().await {
        count += 1;
        if matches!(ev, TranslationEvent::Stopped) {
            break;
        }
    }
    assert_eq!(count, 4);
}
