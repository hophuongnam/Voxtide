use std::path::PathBuf;

use voxtide_core::audio::mock::WavSource;
use voxtide_core::persistence::Store;
use voxtide_core::session::{SessionController, StartArgs};
use voxtide_core::translation::mock::MockProvider;
use voxtide_core::translation::tokens::TranslationStatus;
use voxtide_core::translation::{Mode, SessionConfig, TranslationEvent, WhichLang};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn make_cfg() -> SessionConfig {
    SessionConfig {
        api_key: "test".into(),
        mode: Mode::Conversation,
        language_a: "en".into(),
        language_b: "vi".into(),
        mine: WhichLang::A,
    }
}

fn make_provider() -> Box<MockProvider> {
    Box::new(MockProvider::with_script(vec![
        TranslationEvent::Connected,
        TranslationEvent::Final {
            text: "Hi".into(),
            language: Some("en".into()),
            status: TranslationStatus::Original,
            speaker: None,
            ts_ms: 50,
        },
        TranslationEvent::Stopped,
    ]))
}

#[tokio::test]
async fn second_start_without_stop_returns_already_running() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("v.db")).await.unwrap();
    let ctl = SessionController::new(store);

    // First start: must succeed.
    let first = ctl
        .start(StartArgs {
            cfg: make_cfg(),
            source: Box::new(WavSource::open(&fixture("hello-en-16k-mono.wav"), false).unwrap()),
            provider: make_provider(),
            device_label: None,
        })
        .await;
    assert!(first.is_ok(), "first start should succeed");

    // Second start (no stop in between): must fail with the double-start guard.
    let second = ctl
        .start(StartArgs {
            cfg: make_cfg(),
            source: Box::new(WavSource::open(&fixture("hello-en-16k-mono.wav"), false).unwrap()),
            provider: make_provider(),
            device_label: None,
        })
        .await;

    let err = second.expect_err("second start without stop must return Err");
    assert!(
        err.to_string().contains("already running"),
        "error should mention 'already running', got: {err}"
    );

    // Clean up: stop the first session so the test doesn't leave dangling tasks.
    ctl.stop().await.unwrap();
}
