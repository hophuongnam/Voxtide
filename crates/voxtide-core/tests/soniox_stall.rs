//! STALL regression tests: a wedged network path (TCP connects but the
//! WebSocket handshake / writes never complete — congested uplink, VPN
//! blackhole, half-dead socket) must never park the caller inside
//! `send_audio`, and must surface as the ordinary reconnect ladder instead
//! of hanging silently forever.
//!
//! Field incident (2026-07-02, Zoom running alongside): the provider's
//! background task parked in a timeout-less `connect_async`/`ws_tx.send`,
//! stopped draining the audio channel, and the session worker then parked in
//! `send_audio().await` inside its select arm body — freezing the transcript
//! AND making Stop unresponsive (no SessionStopped ever emitted).

use std::time::Duration;

use tokio::net::TcpListener;
use voxtide_core::translation::soniox::SonioxBYOK;
use voxtide_core::translation::{Mode, SessionConfig, TranslationEvent, TranslationProvider};

fn cfg() -> SessionConfig {
    SessionConfig {
        api_key: "test".into(),
        mode: Mode::Meeting,
        language_a: "en".into(),
        language_b: "vi".into(),
        capture_mic: false,
    }
}

/// A server that accepts TCP connections and then goes silent: it never
/// answers the WebSocket upgrade, modelling a blackholed / stalled network
/// path. The accepted sockets are held open (not dropped) so the client sees
/// an established connection that simply never progresses.
async fn spawn_silent_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let mut held = Vec::new();
        while let Ok((stream, _)) = listener.accept().await {
            held.push(stream);
        }
    });
    format!("ws://{addr}")
}

/// `send_audio` must NEVER block the caller, no matter how wedged the
/// connection is. The session worker calls it inline in its control loop; a
/// blocking send parks the worker where the stop signal can't preempt it —
/// the UI is then stuck recording forever (the field incident).
///
/// Pre-fix: the background task is parked in a timeout-less `connect_async`,
/// never draining the audio channel (capacity 64), so send #65 parks and the
/// per-send timeout below trips.
#[tokio::test]
async fn send_audio_never_blocks_while_connection_is_stalled() {
    let url = spawn_silent_server().await;
    let mut provider = SonioxBYOK::with_endpoint(&url);
    provider.open(cfg()).await.unwrap();

    // 100 frames × 100 ms = 10 s of audio, well past the channel's 64-frame
    // capacity. Every send must return promptly (frames beyond capacity are
    // dropped, not queued — during a stall those words are lost regardless,
    // and bounded latency matters more than a stale backlog).
    for i in 0..100 {
        tokio::time::timeout(
            Duration::from_millis(200),
            provider.send_audio(vec![0u8; 3200]),
        )
        .await
        .unwrap_or_else(|_| panic!("send_audio #{i} blocked on a stalled connection"))
        .unwrap();
    }

    // The task may still be parked in its (bounded) first connect attempt;
    // close() aborts it — a timeout error here is acceptable teardown.
    let _ = provider.close().await;
}

/// A stalled connect must be bounded by a timeout and feed the ordinary
/// reconnect ladder (Reconnecting events, then a terminal Error + Stopped
/// once the budget is spent) — never hang silently forever with the UI
/// showing a live session and no transcript.
///
/// Pre-fix: `connect_async` has no timeout, the silent server never answers,
/// and `next_event()` yields nothing until the outer test timeout trips.
#[tokio::test]
async fn stalled_connect_times_out_and_runs_the_reconnect_ladder() {
    let url = spawn_silent_server().await;
    // 50 ms connect timeout + 1 ms backoff: the full MAX_ATTEMPTS ladder
    // completes in well under a second without sleeping the real schedule.
    let mut provider = SonioxBYOK::with_endpoint_and_backoff(&url, |_| 1)
        .with_connect_timeout(Duration::from_millis(50));
    provider.open(cfg()).await.unwrap();

    tokio::time::timeout(Duration::from_secs(5), async {
        let mut saw_reconnecting = false;
        let mut saw_error = false;
        while let Some(ev) = provider.next_event().await {
            match ev {
                TranslationEvent::Reconnecting { .. } => saw_reconnecting = true,
                TranslationEvent::Error(msg) => {
                    assert!(
                        msg.contains("connect"),
                        "terminal error should name the connect failure, got: {msg}"
                    );
                    saw_error = true;
                }
                TranslationEvent::Stopped => break,
                _ => {}
            }
        }
        assert!(
            saw_reconnecting,
            "a timed-out connect must surface Reconnecting events"
        );
        assert!(
            saw_error,
            "an exhausted ladder must surface a terminal Error before Stopped"
        );
    })
    .await
    .expect("provider must self-terminate, not hang in a timeout-less connect");

    let _ = provider.close().await;
}
