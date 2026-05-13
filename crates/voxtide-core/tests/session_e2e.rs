use std::path::PathBuf;
use std::time::Duration;

use voxtide_core::audio::mock::WavSource;
use voxtide_core::persistence::sessions::Sessions;
use voxtide_core::persistence::tokens::Tokens;
use voxtide_core::persistence::Store;
use voxtide_core::session::{CoreEvent, SessionController, StartArgs};
use voxtide_core::translation::mock::MockProvider;
use voxtide_core::translation::tokens::TranslationStatus;
use voxtide_core::translation::{Mode, SessionConfig, TranslationEvent, WhichLang};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

#[tokio::test]
async fn session_persists_finals_and_emits_events() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("v.db")).await.unwrap();

    let wav = Box::new(WavSource::open(&fixture("hello-en-16k-mono.wav"), false).unwrap());

    let script = vec![
        TranslationEvent::Connected,
        TranslationEvent::Live {
            text: "Hel".into(), language: Some("en".into()),
            status: TranslationStatus::Original, speaker: Some("1".into()),
        },
        TranslationEvent::Final {
            text: "Hello".into(), language: Some("en".into()),
            status: TranslationStatus::Original, speaker: Some("1".into()), ts_ms: 100,
        },
        TranslationEvent::Final {
            text: "Xin chào".into(), language: Some("vi".into()),
            status: TranslationStatus::Translation, speaker: Some("1".into()), ts_ms: 110,
        },
        TranslationEvent::Stopped,
    ];
    let provider = Box::new(MockProvider::with_script(script));

    let ctl = SessionController::new(store);
    let mut events = ctl.subscribe();

    let session_id = ctl.start(StartArgs {
        cfg: SessionConfig {
            api_key: "test".into(), mode: Mode::Meeting,
            language_a: "en".into(), language_b: "vi".into(), mine: WhichLang::B,
        },
        source: wav,
        provider,
        device_label: Some("Mock WAV".into()),
    }).await.unwrap();

    let mut finals_seen = 0;
    let mut got_live = false;
    while let Ok(ev) = tokio::time::timeout(Duration::from_secs(3), events.recv()).await {
        let Ok(ev) = ev else { break; };
        match ev {
            CoreEvent::TranscriptFinal { .. } => finals_seen += 1,
            CoreEvent::TranscriptLive { .. } => got_live = true,
            CoreEvent::SessionStopped { .. } => break,
            _ => {}
        }
    }
    assert!(got_live);
    assert_eq!(finals_seen, 2);

    // Persistence assertions.
    let store = ctl.store();
    let rows = Sessions::list(store.pool(), 10).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, session_id);
    assert!(rows[0].ended_at.is_some());

    let tokens = Tokens::list_by_session(store.pool(), session_id).await.unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].text, "Hello");
    assert_eq!(tokens[1].text, "Xin chào");
}
