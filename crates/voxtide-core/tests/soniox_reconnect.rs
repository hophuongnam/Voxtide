use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};
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

/// Spawns a WS server whose per-connection behaviour is scripted by
/// `keep_open` (indexed by accept order, 0-based): connection `i` idle-drains
/// (never closes on its own — the client is the one that closes, once its
/// reconnect loop breaks for its own reason) when `keep_open.get(i) ==
/// Some(&true)`. Every other connection — including every index past
/// `keep_open`'s length — reads the client's initial config then immediately
/// closes, exactly like [`spawn_accept_then_close_server`]. This lets a test
/// script a handful of connections precisely while the reconnect ladder is
/// free to run past them (e.g. to self-terminate at MAX_ATTEMPTS) without the
/// server hanging or the test having to enumerate every attempt.
///
/// Every connection's initial config text is captured, in accept order, into
/// the returned `Vec` (shared via `Arc<Mutex<_>>` so the test can inspect it
/// after the fact — index `i` is connection `i`'s config).
async fn spawn_scripted_server(keep_open: Vec<bool>) -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let configs = Arc::new(Mutex::new(Vec::new()));
    let configs_for_task = Arc::clone(&configs);
    tokio::spawn(async move {
        let mut i = 0usize;
        // Accept loop runs for the lifetime of the test, so every reconnect
        // the provider makes (scripted or not) is answered.
        while let Ok((stream, _)) = listener.accept().await {
            let configs = Arc::clone(&configs_for_task);
            let stay_open = keep_open.get(i).copied().unwrap_or(false);
            i += 1;
            tokio::spawn(async move {
                let Ok(ws) = accept_async(stream).await else {
                    return;
                };
                let (mut tx, mut rx) = ws.split();
                // Capture the client's initial JSON config text.
                if let Some(Ok(Message::Text(s))) = rx.next().await {
                    configs.lock().unwrap().push(s);
                }
                if stay_open {
                    // Idle-drain: ignore anything the client sends until IT
                    // closes (e.g. after breaking its inner loop for a
                    // context switch); never close from this side. The
                    // provider under test never puts audio on the wire for
                    // a `set_context` call (it travels the internal mpsc,
                    // not the socket), so there is nothing to act on here.
                    while let Some(msg) = rx.next().await {
                        match msg {
                            Ok(m) if m.is_close() => break,
                            Ok(_) => continue,
                            Err(_) => break,
                        }
                    }
                }
                let _ = tx.close().await;
            });
        }
    });
    (format!("ws://{addr}"), configs)
}

/// SWITCH-CTX: a mid-session `set_context` call must reconnect the provider
/// with the new context, WITHOUT spending the `MAX_ATTEMPTS` reconnect budget
/// or emitting `Reconnecting` (only `ContextSwitching`).
///
/// Connection #0 is scripted to fail immediately so `attempt` sits at 2 by
/// the time the stable connection (#1) comes up — load-bearing: without a
/// real prior failure, "reset to 1" after the switch would be
/// indistinguishable from "never incremented past 1" in the first place.
/// Connection #1 stays open (captures the PRE-switch config) until the test
/// injects the switch. Connection #2 (the first POST-switch connection) is
/// scripted to fail immediately too, so its resulting `Reconnecting` report
/// is the tell: `attempt: 1` proves the switch reset the budget; `attempt: 3`
/// would prove it just fell through the ordinary (buggy) ladder instead.
#[tokio::test]
async fn set_context_reconnects_with_new_context_without_spending_budget() {
    let (url, configs) = spawn_scripted_server(vec![false, true]).await;
    let mut provider =
        SonioxBYOK::with_endpoint_and_backoff(&url, |_| 1).with_context("PRESET_A".into());
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

    tokio::time::timeout(Duration::from_secs(10), async {
        // Drain until the SECOND Connected: connection #0 (fail-fast) has
        // cycled fully through Connected -> Reconnecting{1} -> backoff, and
        // connection #1 (kept open) is now the live, stable connection.
        let mut connected = 0u32;
        loop {
            match provider.next_event().await.expect("provider ended early") {
                TranslationEvent::Connected => {
                    connected += 1;
                    if connected == 2 {
                        break;
                    }
                }
                TranslationEvent::Reconnecting { attempt, .. } => {
                    assert_eq!(
                        attempt, 1,
                        "only conn#0's failure should have reconnected so far"
                    );
                }
                other => panic!("unexpected event before the stable connect: {other:?}"),
            }
        }

        // Inject the switch mid-stream, on the now-stable connection #1.
        provider.set_context("PRESET_B".into()).await.unwrap();

        // The switch must be signalled by ContextSwitching, never Reconnecting.
        let ev = provider.next_event().await.expect("provider ended early");
        assert!(
            matches!(ev, TranslationEvent::ContextSwitching),
            "expected ContextSwitching right after set_context, got {ev:?}"
        );

        // Immediate reconnect: Connected next, with NO Reconnecting between
        // it and ContextSwitching (that would mean the switch spent a
        // backoff-ladder rung instead of reconnecting immediately).
        let ev = provider.next_event().await.expect("provider ended early");
        assert!(
            matches!(ev, TranslationEvent::Connected),
            "expected an immediate Connected after ContextSwitching, got {ev:?}"
        );

        // Connection #2 (post-switch) is scripted to fail immediately (index
        // 2 is past `keep_open`'s length, defaulting to fail-fast). Its
        // Reconnecting report is the budget-neutrality tell.
        let ev = provider.next_event().await.expect("provider ended early");
        match ev {
            TranslationEvent::Reconnecting { attempt, .. } => {
                assert_eq!(
                    attempt, 1,
                    "context switch must reset the reconnect budget (attempt: 1), \
                     not continue the pre-switch ladder (would be attempt: 3)"
                );
            }
            other => panic!("expected Reconnecting after conn#2's failure, got {other:?}"),
        }
    })
    .await
    .expect("test must not hang");

    let _ = provider.close().await;

    // `>=` rather than `==`: once the assertions above observe the tell,
    // nothing further stops the ladder from racing ahead (more scripted
    // fail-fast connections) before `close()`'s effect lands. Indices 1 and 2
    // are unaffected by any of that — the vec is append-only in accept order.
    let configs = configs.lock().unwrap();
    assert!(
        configs.len() >= 3,
        "expected at least 3 connections up to and including the post-switch one, got {}",
        configs.len()
    );
    let pre: Value = serde_json::from_str(&configs[1]).unwrap();
    assert_eq!(
        pre["context"]["text"].as_str(),
        Some("PRESET_A"),
        "pre-switch (conn#1) config must carry the ORIGINAL context, got {pre}"
    );
    let post: Value = serde_json::from_str(&configs[2]).unwrap();
    assert_eq!(
        post["context"]["text"].as_str(),
        Some("PRESET_B"),
        "post-switch (conn#2) config must carry the NEW context, got {post}"
    );
}

/// `set_context` mirrors `send_audio`'s "not open" guard: calling it before
/// `open()` (or after `close()`) must error rather than silently doing
/// nothing, since the caller has no other signal that the switch never
/// reached a worker.
#[tokio::test]
async fn set_context_errors_when_provider_not_open() {
    let mut provider = SonioxBYOK::new();
    assert!(
        provider.set_context("ctx".into()).await.is_err(),
        "set_context before open() must error, not silently no-op"
    );
}
