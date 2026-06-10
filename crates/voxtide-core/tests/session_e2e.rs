use std::path::PathBuf;
use std::time::Duration;

use voxtide_core::audio::mock::WavSource;
use voxtide_core::persistence::sessions::Sessions;
use voxtide_core::persistence::tokens::Tokens;
use voxtide_core::persistence::Store;
use voxtide_core::session::{CoreEvent, SessionController, StartArgs};
use voxtide_core::translation::mock::MockProvider;
use voxtide_core::translation::tokens::TranslationStatus;
use voxtide_core::translation::{FinalToken, Mode, SessionConfig, TranslationEvent};

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
        TranslationEvent::Finals {
            tokens: vec![FinalToken {
                text: "Hello".into(),
                language: Some("en".into()),
                status: TranslationStatus::Original,
                speaker: Some("1".into()),
                ts_ms: 100,
            }],
            lag_ms: None,
        },
        // A speech pause. Must be persisted as a break row (not broadcast-only)
        // so replay chunks rows at the same boundaries the live view did.
        TranslationEvent::UtteranceBreak,
        TranslationEvent::Finals {
            tokens: vec![FinalToken {
                text: "Xin chào".into(),
                language: Some("vi".into()),
                status: TranslationStatus::Translation,
                speaker: Some("1".into()),
                ts_ms: 110,
            }],
            lag_ms: None,
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
    // Two finals + one break row. The break row stamps wall-clock now_ms()
    // while the scripted finals persist their tiny ts values verbatim, so
    // ts-ordered listing puts the break LAST here (real sessions interleave
    // correctly — finals carry receive-stamped epoch ts too). Assert
    // position-agnostically; the replay ordering semantics are pinned by the
    // frontend coalesceTokens tests, which control ts.
    assert_eq!(tokens.len(), 3, "two finals + one persisted break row");
    let breaks: Vec<_> = tokens.iter().filter(|t| t.is_break == 1).collect();
    assert_eq!(
        breaks.len(),
        1,
        "the pause must persist as exactly one break row"
    );
    assert_eq!(breaks[0].text, "", "break rows carry no text");
    let finals: Vec<_> = tokens.iter().filter(|t| t.is_break == 0).collect();
    assert_eq!(finals[0].text, "Hello");
    assert_eq!(finals[1].text, "Xin chào");
    // Timestamps are persisted AS RECEIVED — the worker no longer subtracts the
    // session start. The scripted finals carry ts_ms 100 and 110, so those exact
    // values must land in the DB. (Before the fix the worker stored
    // `ts_ms - started_at`, i.e. ~100 minus a wall-clock epoch — a large
    // negative number — which is the bug that made reopened sessions render at
    // the 1970 epoch.)
    assert_eq!(
        finals[0].ts_ms, 100,
        "ts must be persisted unmodified (no subtraction)"
    );
    assert_eq!(
        finals[1].ts_ms, 110,
        "ts must be persisted unmodified (no subtraction)"
    );
}

#[tokio::test]
async fn finals_frame_persists_batch_and_reports_audio_anchored_latency() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("v.db")).await.unwrap();

    let wav = Box::new(WavSource::open(&fixture("hello-en-16k-mono.wav"), false).unwrap());
    // One frame carrying TWO finals and an audio-anchored lag of 800 ms —
    // the shape Soniox actually produces (original + translation finalized
    // in the same WS frame).
    let script = vec![
        TranslationEvent::Connected,
        TranslationEvent::Finals {
            tokens: vec![
                FinalToken {
                    text: "Hello".into(),
                    language: Some("en".into()),
                    status: TranslationStatus::Original,
                    speaker: Some("1".into()),
                    ts_ms: 100,
                },
                FinalToken {
                    text: "Xin chào".into(),
                    language: Some("vi".into()),
                    status: TranslationStatus::Translation,
                    speaker: Some("1".into()),
                    ts_ms: 110,
                },
            ],
            lag_ms: Some(800),
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
            device_label: None,
        })
        .await
        .unwrap();

    let mut finals_broadcast = 0;
    let mut latency_events: Vec<u64> = Vec::new();
    while let Ok(Ok(ev)) = tokio::time::timeout(Duration::from_secs(3), events.recv()).await {
        match ev {
            // The wire contract is unchanged by the internal batching: one
            // TranscriptFinal per token.
            CoreEvent::TranscriptFinal { .. } => finals_broadcast += 1,
            CoreEvent::Latency { median_ms } => latency_events.push(median_ms),
            CoreEvent::SessionStopped { .. } => break,
            _ => {}
        }
    }
    assert_eq!(
        finals_broadcast, 2,
        "one TranscriptFinal broadcast per token"
    );
    assert_eq!(
        latency_events,
        vec![800],
        "the frame's audio-anchored lag must surface as the latency median"
    );

    let tokens = Tokens::list_by_session(ctl.store().pool(), session_id)
        .await
        .unwrap();
    assert_eq!(tokens.len(), 2, "the whole finals batch must persist");
    assert_eq!(tokens[0].text, "Hello");
    assert_eq!(tokens[1].text, "Xin chào");
}

#[tokio::test]
async fn latency_emission_is_throttled_to_one_per_second() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("v.db")).await.unwrap();

    let wav = Box::new(WavSource::open(&fixture("hello-en-16k-mono.wav"), false).unwrap());
    let one = |text: &str, ts_ms: i64, lag: u64| TranslationEvent::Finals {
        tokens: vec![FinalToken {
            text: text.into(),
            language: Some("en".into()),
            status: TranslationStatus::Original,
            speaker: Some("1".into()),
            ts_ms,
        }],
        lag_ms: Some(lag),
    };
    // Three finals frames in immediate succession: only the FIRST may emit a
    // CoreEvent::Latency — the rest land inside the 1 s throttle window.
    let script = vec![
        TranslationEvent::Connected,
        one("a", 100, 800),
        one("b", 110, 900),
        one("c", 120, 1000),
        TranslationEvent::Stopped,
    ];
    let provider = Box::new(MockProvider::with_script(script));

    let ctl = SessionController::new(store);
    let mut events = ctl.subscribe();

    ctl.start(StartArgs {
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

    let mut latency_events: Vec<u64> = Vec::new();
    while let Ok(Ok(ev)) = tokio::time::timeout(Duration::from_secs(3), events.recv()).await {
        match ev {
            CoreEvent::Latency { median_ms } => latency_events.push(median_ms),
            CoreEvent::SessionStopped { .. } => break,
            _ => {}
        }
    }
    assert_eq!(
        latency_events,
        vec![800],
        "a burst of finals frames must emit at most one Latency per second \
         (the first), not one per frame"
    );
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
        TranslationEvent::Finals {
            tokens: vec![FinalToken {
                text: "Hello".into(),
                language: Some("en".into()),
                status: TranslationStatus::Original,
                speaker: Some("1".into()),
                ts_ms: 100,
            }],
            lag_ms: None,
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
async fn provider_error_is_surfaced_then_session_finalizes() {
    // The most common BYOK failure: Soniox rejects the API key. The provider
    // emits `TranslationEvent::Error(msg)` followed by `Stopped`. Subscribers
    // must receive a `CoreEvent::Error { message }` carrying the detail BEFORE
    // the terminal `SessionStopped`, and the session row must still finalize.
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("v.db")).await.unwrap();

    let wav = Box::new(WavSource::open(&fixture("hello-en-16k-mono.wav"), false).unwrap());
    let provider = Box::new(MockProvider::with_script(vec![
        TranslationEvent::Connected,
        TranslationEvent::Error("Soniox error 401: bad key".into()),
        TranslationEvent::Stopped,
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

    let mut error_msg: Option<String> = None;
    let mut got_stopped = false;
    while let Ok(Ok(ev)) = tokio::time::timeout(Duration::from_secs(3), events.recv()).await {
        match ev {
            CoreEvent::Error { message } => {
                assert!(
                    !got_stopped,
                    "CoreEvent::Error must arrive BEFORE SessionStopped"
                );
                error_msg = Some(message);
            }
            CoreEvent::SessionStopped { .. } => {
                got_stopped = true;
                break;
            }
            _ => {}
        }
    }

    let message = error_msg.expect("a CoreEvent::Error must be broadcast on provider failure");
    assert!(
        message.contains("401"),
        "error message should carry the provider detail, got: {message}"
    );
    assert!(got_stopped, "session must still emit SessionStopped");

    // The session row finalizes despite the error (no orphaned ended_at IS NULL row).
    let row = Sessions::get(ctl.store().pool(), session_id)
        .await
        .unwrap()
        .unwrap();
    assert!(
        row.ended_at.is_some(),
        "session must be finalized after a provider error"
    );
}

#[tokio::test]
async fn explicit_stop_drains_eos_flush_so_trailing_finals_persist() {
    // EOS-TAIL-DROP regression. On explicit stop() the worker breaks via the
    // biased stop arm; before the fix it called provider.close() WITHOUT
    // draining the provider's EOS flush, so Soniox's trailing finals (the last
    // words spoken before stop) never reached the DB or the UI. The worker must
    // now call eos(), drain the flushed finals (persist + broadcast), and only
    // then close — so the tail survives. The two tail finals must also be
    // broadcast BEFORE the terminal SessionStopped.
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("v.db")).await.unwrap();

    // Long-running source so the worker is genuinely parked on next_event() at
    // the moment we call stop(), exercising the real stop→drain ordering.
    let wav = Box::new(WavSource::open(&fixture("hello-en-16k-mono.wav"), true).unwrap());

    // Script: a live + a final delivered up front. flush_on_eos: two trailing
    // finals plus the terminal Stopped, released ONLY when eos() is called.
    let script = vec![
        TranslationEvent::Connected,
        TranslationEvent::Live {
            text: "Goodbye wor".into(),
            language: Some("en".into()),
            status: TranslationStatus::Original,
            speaker: Some("1".into()),
        },
        TranslationEvent::Finals {
            tokens: vec![FinalToken {
                text: "script final".into(),
                language: Some("en".into()),
                status: TranslationStatus::Original,
                speaker: Some("1".into()),
                ts_ms: 100,
            }],
            lag_ms: None,
        },
    ];
    let flush_on_eos = vec![
        TranslationEvent::Finals {
            tokens: vec![FinalToken {
                text: "tail one".into(),
                language: Some("en".into()),
                status: TranslationStatus::Original,
                speaker: Some("1".into()),
                ts_ms: 200,
            }],
            lag_ms: None,
        },
        TranslationEvent::Finals {
            tokens: vec![FinalToken {
                text: "tail two".into(),
                language: Some("vi".into()),
                status: TranslationStatus::Translation,
                speaker: Some("1".into()),
                ts_ms: 210,
            }],
            lag_ms: None,
        },
        TranslationEvent::Stopped,
    ];
    let provider = Box::new(MockProvider::with_script_and_flush(script, flush_on_eos));

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

    // Wait until the up-front script final has been broadcast (and thus
    // persisted) so we know stop() can't race ahead of the pre-stop events.
    loop {
        let ev = tokio::time::timeout(Duration::from_secs(3), events.recv())
            .await
            .expect("script final should arrive")
            .expect("broadcast channel open");
        if matches!(ev, CoreEvent::TranscriptFinal { ref text, .. } if text == "script final") {
            break;
        }
    }

    // Explicit stop: must trigger the EOS drain.
    ctl.stop().await.unwrap();

    // Record the order of trailing finals vs. SessionStopped.
    let mut tail_finals_before_stop: Vec<String> = Vec::new();
    let mut got_stopped = false;
    while let Ok(Ok(ev)) = tokio::time::timeout(Duration::from_secs(3), events.recv()).await {
        match ev {
            CoreEvent::TranscriptFinal { text, .. } => {
                assert!(
                    !got_stopped,
                    "trailing final '{text}' arrived AFTER SessionStopped"
                );
                tail_finals_before_stop.push(text);
            }
            CoreEvent::SessionStopped { .. } => {
                got_stopped = true;
                break;
            }
            _ => {}
        }
    }

    assert!(got_stopped, "session must emit SessionStopped");
    assert_eq!(
        tail_finals_before_stop,
        vec!["tail one".to_string(), "tail two".to_string()],
        "both EOS-flushed finals must broadcast (in order) before SessionStopped"
    );

    // Persistence: the script final AND both tail finals must be in the DB.
    let tokens = Tokens::list_by_session(ctl.store().pool(), session_id)
        .await
        .unwrap();
    let texts: Vec<&str> = tokens.iter().map(|t| t.text.as_str()).collect();
    assert!(
        texts.contains(&"tail one") && texts.contains(&"tail two"),
        "EOS-flushed trailing finals must be persisted, got: {texts:?}"
    );
    assert!(
        texts.contains(&"script final"),
        "pre-stop final must still be persisted, got: {texts:?}"
    );

    let row = Sessions::get(ctl.store().pool(), session_id)
        .await
        .unwrap()
        .unwrap();
    assert!(row.ended_at.is_some(), "session must be finalized");
}

#[tokio::test]
async fn audio_source_ending_triggers_eos_and_finalizes_session() {
    // MIC-DEVICE-LOSS regression. When the audio source ends on its own — a WAV
    // hitting EOF, or (in production) a mic/loopback device being unplugged so
    // the capture thread drops its stream and the frame channel closes — the
    // worker's audio arm yields `None` (`audio_done` flips true). Before the fix
    // the worker simply stopped polling that arm and idled forever waiting on a
    // `Stopped` the provider would never send on its own, leaving a zombie
    // session: UI stuck REC, row stuck `ended_at IS NULL`, no recovery short of
    // a manual stop. The worker must now call `provider.eos()` when the source
    // drains so the provider flushes and emits `Stopped`, the loop breaks, and
    // the session finalizes — all WITHOUT any explicit `stop()`.
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("v.db")).await.unwrap();

    // Non-realtime WAV: drains quickly, closing its frame channel at EOF — the
    // exact channel-close shape a real device-loss produces.
    let wav = Box::new(WavSource::open(&fixture("hello-en-16k-mono.wav"), false).unwrap());

    // Script WITHOUT a terminal Stopped. `Stopped` is queued in flush_on_eos, so
    // the provider releases it ONLY when eos() is called. Per MockProvider's
    // contract, the retained flush sender keeps next_event() parked forever until
    // eos() fires — so if the worker never calls eos() on audio-drain, no
    // SessionStopped is ever broadcast. The INNER 5s recv timeout below then
    // expires first, dropping out of the receive loop with got_stopped == false,
    // and the `assert!(got_stopped)` fails — that is the RED signal. The outer
    // 10s only guards a hang in start()/DB ops, which have no inner timeout.
    let script = vec![
        TranslationEvent::Connected,
        TranslationEvent::Finals {
            tokens: vec![FinalToken {
                text: "Hello".into(),
                language: Some("en".into()),
                status: TranslationStatus::Original,
                speaker: Some("1".into()),
                ts_ms: 100,
            }],
            lag_ms: None,
        },
    ];
    let flush_on_eos = vec![TranslationEvent::Stopped];
    let provider = Box::new(MockProvider::with_script_and_flush(script, flush_on_eos));

    let ctl = SessionController::new(store);
    let mut events = ctl.subscribe();

    // Outer guard for start()/DB hangs only (those have no inner timeout). The
    // missed-eos RED path is caught by the inner 5s recv timeout → got_stopped
    // assert failure, not by this 10s budget.
    tokio::time::timeout(Duration::from_secs(10), async {
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

        // We deliberately do NOT call ctl.stop(). The session must terminate on
        // its own once the source drains.
        let mut got_stopped = false;
        while let Ok(Ok(ev)) = tokio::time::timeout(Duration::from_secs(5), events.recv()).await {
            if matches!(ev, CoreEvent::SessionStopped { .. }) {
                got_stopped = true;
                break;
            }
        }
        assert!(
            got_stopped,
            "SessionStopped must fire when the audio source ends, without an explicit stop()"
        );

        let row = Sessions::get(ctl.store().pool(), session_id)
            .await
            .unwrap()
            .unwrap();
        assert!(
            row.ended_at.is_some(),
            "session must be finalized (ended_at set) when the audio source ends"
        );
        assert!(row.duration_ms.is_some());
    })
    .await
    .expect("start()/DB must not hang for 10s; the missed-eos case fails via the inner assert");
}

#[tokio::test]
async fn active_session_id_tracks_running_state() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("v.db")).await.unwrap();

    let wav = Box::new(WavSource::open(&fixture("hello-en-16k-mono.wav"), false).unwrap());
    let provider = Box::new(MockProvider::with_script(vec![
        TranslationEvent::Connected,
        TranslationEvent::Finals {
            tokens: vec![FinalToken {
                text: "Hello".into(),
                language: Some("en".into()),
                status: TranslationStatus::Original,
                speaker: Some("1".into()),
                ts_ms: 100,
            }],
            lag_ms: None,
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
