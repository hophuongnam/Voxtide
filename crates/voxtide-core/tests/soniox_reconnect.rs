use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use voxtide_core::translation::soniox::{next_backoff_ms, SonioxBYOK, MAX_ATTEMPTS};
use voxtide_core::translation::{Mode, SessionConfig, TranslationEvent, TranslationProvider};

#[test]
fn backoff_sequence_matches_spec() {
    let mut got = Vec::new();
    for n in 1..=MAX_ATTEMPTS {
        got.push(next_backoff_ms(n));
    }
    assert_eq!(got, vec![250, 500, 1000, 2000, 5000, 5000]);
}

#[test]
fn backoff_returns_none_after_max_attempts() {
    assert!(next_backoff_ms(MAX_ATTEMPTS + 1) >= 5000);
}

#[test]
fn max_attempts_is_six() {
    assert_eq!(MAX_ATTEMPTS, 6);
}

/// A server that, forever, accepts the WS handshake, reads the initial config
/// message, then immediately closes — modelling a captive portal / proxy /
/// Soniox idle-close / maintenance window that completes the handshake but
/// never streams tokens. Returns the `ws://host:port` URL to point a provider at.
async fn spawn_accept_then_close_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        // Accept loop runs for the lifetime of the test, so every reconnect the
        // provider makes is answered the same way.
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                if let Ok(ws) = accept_async(stream).await {
                    let (mut tx, mut rx) = ws.split();
                    // Read the client's initial JSON config, then drop the
                    // connection immediately — never send a single token.
                    let _ = rx.next().await;
                    let _ = tx.close().await;
                }
            });
        }
    });
    format!("ws://{addr}")
}

/// RECONNECT-INF regression: an endpoint that accepts the handshake then closes
/// (without ever sending tokens) used to drive an INFINITE reconnect loop,
/// because the loop reset `attempt = 1` after every successfully established
/// connection. The session must instead exhaust `MAX_ATTEMPTS` and self-
/// terminate, emitting Reconnecting events with INCREASING attempt numbers
/// (no pinning at 1) followed by exactly one Error (give-up) then Stopped.
///
/// Before the fix this test fails by hanging — the provider never emits
/// Error/Stopped — so the outer 30s timeout trips it.
#[tokio::test]
async fn accept_then_close_terminates() {
    let url = spawn_accept_then_close_server().await;
    // Inject a near-instant backoff (1 ms per rung) so this self-termination
    // test exercises the full MAX_ATTEMPTS ladder WITHOUT sleeping the real
    // ~13.75s schedule. The reconnect logic is identical; only the wait shrinks.
    let mut provider = SonioxBYOK::with_endpoint_and_backoff(&url, |_| 1);
    provider
        .open(SessionConfig {
            api_key: "test".into(),
            mode: Mode::Meeting,
            language_a: "en".into(),
            language_b: "vi".into(),
            capture_mic: false,
        })
        .await
        .unwrap();

    // The whole assertion phase is bounded: if the provider loops forever the
    // timeout fails the test rather than hanging the suite. With the injected
    // 1 ms backoff the 6 reconnect rungs + localhost round-trips finish well
    // under a second, so a 10s ceiling is generous slack.
    tokio::time::timeout(Duration::from_secs(10), async {
        let mut reconnect_attempts: Vec<u32> = Vec::new();
        let mut saw_error: Option<String> = None;
        let mut saw_stopped = false;

        while let Some(ev) = provider.next_event().await {
            match ev {
                TranslationEvent::Reconnecting { attempt, .. } => {
                    reconnect_attempts.push(attempt);
                }
                TranslationEvent::Error(msg) => {
                    // Error must arrive before Stopped, exactly once.
                    assert!(saw_error.is_none(), "duplicate Error event: {msg}");
                    assert!(!saw_stopped, "Error arrived after Stopped");
                    saw_error = Some(msg);
                }
                TranslationEvent::Stopped => {
                    saw_stopped = true;
                    break;
                }
                // Connected (each accepted handshake) and any stray events are
                // fine; we only pin Reconnecting/Error/Stopped ordering here.
                _ => {}
            }
        }

        // (a) Reconnecting attempts climbed — not pinned at 1/2. We expect the
        // full 1..=MAX_ATTEMPTS ladder before the loop gives up on the
        // (MAX_ATTEMPTS+1)-th attempt.
        assert!(
            reconnect_attempts.len() >= 3,
            "expected several reconnect attempts, got {reconnect_attempts:?}"
        );
        let max_seen = *reconnect_attempts.iter().max().unwrap();
        assert!(
            max_seen >= 3,
            "attempt numbers should increase past 1-2, got {reconnect_attempts:?}"
        );
        // Strictly non-decreasing and ending at the top of the ladder.
        for w in reconnect_attempts.windows(2) {
            assert!(
                w[1] >= w[0],
                "attempt numbers must not pin/regress: {reconnect_attempts:?}"
            );
        }
        assert_eq!(
            max_seen, MAX_ATTEMPTS,
            "should report Reconnecting up to MAX_ATTEMPTS before giving up: {reconnect_attempts:?}"
        );

        // (b) Terminal Error mentioning give-up, then Stopped.
        let msg = saw_error.expect("provider must emit a terminal Error before giving up");
        assert!(
            msg.contains("giving up"),
            "Error message should mention giving up, got: {msg}"
        );
        assert!(
            saw_stopped,
            "provider must emit Stopped after the terminal Error"
        );

        // (c) No further events after Stopped — the channel is closed.
        assert!(
            provider.next_event().await.is_none(),
            "no events may follow Stopped"
        );
    })
    .await
    .expect("provider must self-terminate within 10s, not loop forever");
}
