use std::path::PathBuf;
use std::time::Duration;

use voxtide_core::audio::mock::WavSource;
use voxtide_core::persistence::sessions::Sessions;
use voxtide_core::persistence::tokens::Tokens;
use voxtide_core::persistence::Store;
use voxtide_core::session::{CoreEvent, SessionController, StartArgs};
use voxtide_core::translation::mock::MockProvider;
use voxtide_core::translation::tokens::TranslationStatus;
use voxtide_core::translation::{Mode, SessionConfig, TranslationEvent};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[tokio::test]
async fn session_persists_finals_and_emits_events() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("v.db")).await.unwrap();

    let wav = Box::new(WavSource::open(&fixture("hello-en-16k-mono.wav"), false).unwrap());

    let script = vec![
        TranslationEvent::Connected,
        TranslationEvent::Live {
            text: "Hel".into(),
            language: Some("en".into()),
            status: TranslationStatus::Original,
            speaker: Some("1".into()),
        },
        TranslationEvent::Final {
            text: "Hello".into(),
            language: Some("en".into()),
            status: TranslationStatus::Original,
            speaker: Some("1".into()),
            ts_ms: 100,
        },
        TranslationEvent::Final {
            text: "Xin chào".into(),
            language: Some("vi".into()),
            status: TranslationStatus::Translation,
            speaker: Some("1".into()),
            ts_ms: 110,
        },
        TranslationEvent::Stopped,
    ];
    let provider = Box::new(MockProvider::with_script(script));

    let ctl = SessionController::new(store);
    let mut events = ctl.subscribe();

    let session_id = ctl
        .start(StartArgs {
            cfg: SessionConfig {
                api_key: "test".into(),
                mode: Mode::Meeting,
                language_a: "en".into(),
                language_b: "vi".into(),
            },
            source: wav,
            provider,
            device_label: Some("Mock WAV".into()),
        })
        .await
        .unwrap();

    let mut finals_seen = 0;
    let mut got_live = false;
    while let Ok(ev) = tokio::time::timeout(Duration::from_secs(3), events.recv()).await {
        let Ok(ev) = ev else {
            break;
        };
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

    let tokens = Tokens::list_by_session(store.pool(), session_id)
        .await
        .unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].text, "Hello");
    assert_eq!(tokens[1].text, "Xin chào");
}

#[tokio::test]
async fn provider_stream_ending_without_stopped_still_finalizes_session() {
    // Soniox can drop the websocket (server close / network blip / auth expiry)
    // so the stream ends with no terminal `Stopped` event. The session must
    // still be finalized — otherwise the row is stuck `ended_at IS NULL`
    // forever (red "recording" dot, no delete button).
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("v.db")).await.unwrap();

    let wav = Box::new(WavSource::open(&fixture("hello-en-16k-mono.wav"), false).unwrap());
    // NOTE: deliberately NO TranslationEvent::Stopped at the end.
    let provider = Box::new(MockProvider::with_script(vec![
        TranslationEvent::Connected,
        TranslationEvent::Final {
            text: "Hello".into(),
            language: Some("en".into()),
            status: TranslationStatus::Original,
            speaker: Some("1".into()),
            ts_ms: 100,
        },
    ]));

    let ctl = SessionController::new(store);
    let mut events = ctl.subscribe();

    let session_id = ctl
        .start(StartArgs {
            cfg: SessionConfig {
                api_key: "test".into(),
                mode: Mode::Meeting,
                language_a: "en".into(),
                language_b: "vi".into(),
            },
            source: wav,
            provider,
            device_label: None,
        })
        .await
        .unwrap();

    let mut got_stopped = false;
    while let Ok(Ok(ev)) = tokio::time::timeout(Duration::from_secs(3), events.recv()).await {
        if matches!(ev, CoreEvent::SessionStopped { .. }) {
            got_stopped = true;
            break;
        }
    }
    assert!(
        got_stopped,
        "SessionStopped must fire even with no terminal Stopped event"
    );

    let row = Sessions::get(ctl.store().pool(), session_id)
        .await
        .unwrap()
        .unwrap();
    assert!(
        row.ended_at.is_some(),
        "session must be finalized (ended_at set) on any worker-loop exit"
    );
    assert!(row.duration_ms.is_some());
}

#[tokio::test]
async fn active_session_id_tracks_running_state() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("v.db")).await.unwrap();

    let wav = Box::new(WavSource::open(&fixture("hello-en-16k-mono.wav"), false).unwrap());
    let provider = Box::new(MockProvider::with_script(vec![
        TranslationEvent::Connected,
        TranslationEvent::Final {
            text: "Hello".into(),
            language: Some("en".into()),
            status: TranslationStatus::Original,
            speaker: Some("1".into()),
            ts_ms: 100,
        },
        TranslationEvent::Stopped,
    ]));

    let ctl = SessionController::new(store);
    let mut events = ctl.subscribe();

    assert_eq!(ctl.active_session_id(), None, "idle before start");

    let session_id = ctl
        .start(StartArgs {
            cfg: SessionConfig {
                api_key: "test".into(),
                mode: Mode::Meeting,
                language_a: "en".into(),
                language_b: "vi".into(),
            },
            source: wav,
            provider,
            device_label: None,
        })
        .await
        .unwrap();

    assert_eq!(
        ctl.active_session_id(),
        Some(session_id),
        "running after start()"
    );

    while let Ok(ev) = tokio::time::timeout(Duration::from_secs(3), events.recv()).await {
        let Ok(ev) = ev else {
            break;
        };
        if matches!(ev, CoreEvent::SessionStopped { .. }) {
            break;
        }
    }

    // The worker transitions RunState back to Idle after SessionStopped fires.
    // Give it a couple of scheduler ticks to settle before asserting.
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(ctl.active_session_id(), None, "idle again after stop");
}
