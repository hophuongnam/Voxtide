//! Task C2 of the mid-session context switch feature: proves
//! `SessionController::update_context` actually reaches a running session's
//! `provider.set_context`, end to end through the controller's control
//! channel — not just that the provider trait method exists.
//!
//! See docs/superpowers/specs/2026-07-01-mid-session-context-switch-design.md.

use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use tokio::sync::{mpsc, oneshot};

use voxtide_core::audio::{AudioSource, AudioStream};
use voxtide_core::persistence::Store;
use voxtide_core::session::{CoreEvent, SessionController, StartArgs};
use voxtide_core::translation::{Mode, SessionConfig, TranslationEvent, TranslationProvider};
use voxtide_core::Result;

/// Audio source whose frame channel is kept open (the sender is neither
/// dropped nor ever sent on) until the controller signals `stop()`. Real
/// audio content is irrelevant to this test — only the control-channel
/// routing is under test — so the worker's audio arm simply parks on
/// `recv()` forever instead of racing a real source's natural EOF (which
/// would call `provider.eos()` and end the session) against the
/// `update_context` call this test needs to make while the session is still
/// live.
struct StallingSource;

impl AudioSource for StallingSource {
    fn start(&self) -> Result<AudioStream> {
        let (tx, rx) = voxtide_core::audio::channel();
        let (stop_tx, stop_rx) = oneshot::channel::<()>();
        tokio::spawn(async move {
            // Hold `tx` alive until told to stop, then drop it — never send.
            let _ = stop_rx.await;
            drop(tx);
        });
        Ok(AudioStream { rx, stop: stop_tx })
    }
    fn label(&self) -> &str {
        "stalling-test-source"
    }
}

/// Test-only provider whose `set_context` (a) records every text it
/// receives into a shared `Vec` — `MockProvider`'s default no-op
/// `set_context` leaves nothing to observe, so this override is REQUIRED for
/// the assertion below to mean anything — and (b) mirrors `SonioxBYOK`'s real
/// behaviour by emitting `TranslationEvent::ContextSwitching` on its own
/// event stream, so the same test can also confirm the switch surfaces
/// through `session.rs`'s `ContextSwitching -> CoreEvent::ConnectionState`
/// mapping end to end through the controller.
struct ContextRecordingProvider {
    contexts: Arc<StdMutex<Vec<String>>>,
    tx: Option<mpsc::Sender<TranslationEvent>>,
    rx: Option<mpsc::Receiver<TranslationEvent>>,
}

impl ContextRecordingProvider {
    fn new(contexts: Arc<StdMutex<Vec<String>>>) -> Self {
        Self {
            contexts,
            tx: None,
            rx: None,
        }
    }
}

#[async_trait::async_trait]
impl TranslationProvider for ContextRecordingProvider {
    async fn open(&mut self, _cfg: SessionConfig) -> Result<()> {
        let (tx, rx) = mpsc::channel(64);
        let _ = tx.send(TranslationEvent::Connected).await;
        self.tx = Some(tx);
        self.rx = Some(rx);
        Ok(())
    }
    async fn send_audio(&mut self, _pcm: Vec<u8>) -> Result<()> {
        Ok(())
    }
    async fn set_context(&mut self, text: String) -> Result<()> {
        self.contexts.lock().unwrap().push(text);
        if let Some(tx) = &self.tx {
            let _ = tx.send(TranslationEvent::ContextSwitching).await;
        }
        Ok(())
    }
    async fn next_event(&mut self) -> Option<TranslationEvent> {
        self.rx.as_mut()?.recv().await
    }
    async fn eos(&mut self) {
        // Idempotent via `take()`: a second call is a no-op, matching the
        // trait's contract.
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(TranslationEvent::Stopped).await;
        }
    }
    async fn close(&mut self) -> Result<()> {
        self.rx = None;
        self.tx = None;
        Ok(())
    }
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

/// Waits (bounded) for a broadcast `ConnectionState` event carrying the given
/// `state` tag, ignoring every other event in between.
async fn wait_for_connection_state(
    events: &mut tokio::sync::broadcast::Receiver<CoreEvent>,
    want: &str,
) {
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            match events
                .recv()
                .await
                .expect("broadcast channel must stay open")
            {
                CoreEvent::ConnectionState { state, .. } if state == want => break,
                _ => continue,
            }
        }
    })
    .await
    .unwrap_or_else(|_| panic!("timed out waiting for ConnectionState{{state:\"{want}\"}}"));
}

#[tokio::test]
async fn update_context_routes_to_provider_set_context() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("v.db")).await.unwrap();
    let contexts = Arc::new(StdMutex::new(Vec::new()));

    let ctl = SessionController::new(store);
    let mut events = ctl.subscribe();

    ctl.start(StartArgs {
        cfg: make_cfg(),
        source: Box::new(StallingSource),
        provider: Box::new(ContextRecordingProvider::new(contexts.clone())),
        device_label: None,
    })
    .await
    .expect("start should succeed");

    // Let the worker's select loop come fully up (Connected already
    // processed into ConnectionState{state:"active"}) before driving the
    // control channel, so the test isn't racing the worker's first
    // iteration. (Not strictly required for correctness — the mpsc send
    // below just buffers until the worker gets to it — but it keeps the
    // test's intent explicit and deterministic.)
    wait_for_connection_state(&mut events, "active").await;

    ctl.update_context("hello".to_string()).await;

    // The switch must surface through session.rs's
    // ContextSwitching -> ConnectionState{state:"context-switching"} mapping.
    wait_for_connection_state(&mut events, "context-switching").await;

    // The core assertion: `provider.set_context` was actually invoked by the
    // running session with the switched text. `set_context` runs inside the
    // worker task, asynchronously w.r.t. this test task, so poll (bounded)
    // rather than assert immediately.
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if contexts.lock().unwrap().iter().any(|t| t == "hello") {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("provider.set_context(\"hello\") must have been called by the running session");

    ctl.stop().await.unwrap();
}

/// `update_context` on an idle controller (no active session) must be a
/// harmless, promptly-returning no-op — not a panic, not a hang — matching
/// the design's "a late switch is ignored… Controller no-ops when no session
/// is active."
#[tokio::test]
async fn update_context_is_a_noop_when_no_session_running() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("v.db")).await.unwrap();
    let ctl = SessionController::new(store);

    tokio::time::timeout(Duration::from_secs(3), ctl.update_context("hello".into()))
        .await
        .expect("update_context on an idle controller must return promptly, not hang");
}
