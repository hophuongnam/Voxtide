use std::path::PathBuf;
use std::sync::Arc;

use voxtide_core::audio::mock::WavSource;
use voxtide_core::audio::{AudioSource, AudioStream};
use voxtide_core::persistence::sessions::Sessions;
use voxtide_core::persistence::Store;
use voxtide_core::session::{CoreEvent, SessionController, StartArgs};
use voxtide_core::translation::mock::MockProvider;
use voxtide_core::translation::tokens::TranslationStatus;
use voxtide_core::translation::{FinalToken, Mode, SessionConfig, TranslationEvent};
use voxtide_core::{Error, Result};

/// Audio source whose `start()` always fails, simulating a denied mic / TCC
/// screen-recording rejection. Used to prove `start()` performs no irreversible
/// work (no DB row, no `SessionStarted` broadcast) before the fallible source
/// init succeeds.
struct FailingSource;

impl AudioSource for FailingSource {
    fn start(&self) -> Result<AudioStream> {
        Err(Error::Audio("simulated source start failure".into()))
    }
    fn label(&self) -> &str {
        "failing"
    }
}

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
        capture_mic: false,
    }
}

fn make_provider() -> Box<MockProvider> {
    Box::new(MockProvider::with_script(vec![
        TranslationEvent::Connected,
        TranslationEvent::Finals {
            tokens: vec![FinalToken {
                text: "Hi".into(),
                language: Some("en".into()),
                status: TranslationStatus::Original,
                speaker: None,
                ts_ms: 50,
            }],
            lag_ms: None,
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

/// Drives two concurrent `start()` futures via `tokio::join!` and asserts that exactly one
/// succeeds and the other returns the "already running" error. This is the definitive test for
/// the TOCTOU fix: before the tri-state `RunState` guard, both futures could pass the original
/// `is_some()` check simultaneously and both proceed.
#[tokio::test]
async fn concurrent_starts_exactly_one_ok_one_already_running() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("v.db")).await.unwrap();
    // Wrap in Arc so both futures can share the controller.
    let ctl = Arc::new(SessionController::new(store));

    let ctl_a = ctl.clone();
    let ctl_b = ctl.clone();

    let fut_a = async move {
        ctl_a
            .start(StartArgs {
                cfg: make_cfg(),
                source: Box::new(
                    WavSource::open(&fixture("hello-en-16k-mono.wav"), false).unwrap(),
                ),
                provider: make_provider(),
                device_label: None,
            })
            .await
    };

    let fut_b = async move {
        ctl_b
            .start(StartArgs {
                cfg: make_cfg(),
                source: Box::new(
                    WavSource::open(&fixture("hello-en-16k-mono.wav"), false).unwrap(),
                ),
                provider: make_provider(),
                device_label: None,
            })
            .await
    };

    let (res_a, res_b) = tokio::join!(fut_a, fut_b);

    // Exactly one must succeed and one must fail with "already running".
    let ok_count = [res_a.is_ok(), res_b.is_ok()]
        .iter()
        .filter(|&&b| b)
        .count();
    let err_count = [res_a.is_err(), res_b.is_err()]
        .iter()
        .filter(|&&b| b)
        .count();

    assert_eq!(ok_count, 1, "exactly one start should succeed");
    assert_eq!(err_count, 1, "exactly one start should fail");

    // Verify the error carries the expected message.
    let err_msg = if let Err(e) = res_a {
        e.to_string()
    } else if let Err(e) = res_b {
        e.to_string()
    } else {
        unreachable!("err_count == 1 guaranteed one of res_a/res_b is Err");
    };
    assert!(
        err_msg.contains("already running"),
        "error should mention 'already running', got: {err_msg}"
    );

    // Clean up.
    ctl.stop().await.unwrap();
}

/// A failed `start()` (source init fails — denied mic, TCC screen-recording
/// rejection) must leave NO orphan row (`ended_at IS NULL`) and must NOT have
/// broadcast `SessionStarted`. Otherwise the frontend latches `recording=true`
/// with no compensating `SessionStopped` (stuck rec dot / timer) and the row
/// lingers `ended_at IS NULL` until the next launch. The controller must also
/// remain reusable: a subsequent start with a working source succeeds.
#[tokio::test]
async fn failed_start_leaves_no_orphan_row_and_no_started_event() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("v.db")).await.unwrap();
    let ctl = SessionController::new(store);
    let mut events = ctl.subscribe();

    // Start with a source whose start() fails: must return Err.
    let res = ctl
        .start(StartArgs {
            cfg: make_cfg(),
            source: Box::new(FailingSource),
            provider: make_provider(),
            device_label: None,
        })
        .await;
    assert!(res.is_err(), "start with a failing source must return Err");

    // No event may have been broadcast — in particular no SessionStarted.
    match events.try_recv() {
        Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {}
        other => panic!("expected no broadcast event after failed start, got: {other:?}"),
    }

    // No orphan row (ended_at IS NULL) may exist. SELECT COUNT(*) WHERE
    // ended_at IS NULL == 0, expressed through the public API.
    let orphans = Sessions::list(ctl.store().pool(), 100)
        .await
        .unwrap()
        .into_iter()
        .filter(|r| r.ended_at.is_none())
        .count();
    assert_eq!(
        orphans, 0,
        "failed start must leave no ended_at IS NULL row"
    );

    // The controller must still be usable: a subsequent start with a working
    // source succeeds (the slot was released back to Idle).
    let session_id = ctl
        .start(StartArgs {
            cfg: make_cfg(),
            source: Box::new(WavSource::open(&fixture("hello-en-16k-mono.wav"), false).unwrap()),
            provider: make_provider(),
            device_label: None,
        })
        .await
        .expect("controller must be reusable after a failed start");

    // Drain until the working session stops, then confirm exactly one finalized
    // row and zero orphans overall.
    while let Ok(Ok(ev)) =
        tokio::time::timeout(std::time::Duration::from_secs(3), events.recv()).await
    {
        if matches!(ev, CoreEvent::SessionStopped { .. }) {
            break;
        }
    }
    let rows = Sessions::list(ctl.store().pool(), 100).await.unwrap();
    assert_eq!(
        rows.len(),
        1,
        "only the successful session row should exist"
    );
    assert_eq!(rows[0].id, session_id);
    assert!(
        rows[0].ended_at.is_some(),
        "the successful session must be finalized"
    );

    ctl.stop().await.unwrap();
}
