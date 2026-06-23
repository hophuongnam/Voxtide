use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use voxtide_core::translation::soniox::SonioxBYOK;
use voxtide_core::translation::{Mode, SessionConfig, TranslationEvent, TranslationProvider};

async fn spawn_replay_server(replay_path: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let path = replay_path.to_string();
            tokio::spawn(async move {
                let ws = accept_async(stream).await.unwrap();
                let (mut tx, mut rx) = ws.split();
                // 1) Wait for the initial JSON config from the client.
                let _ = rx.next().await;
                // 2) Replay the file, sleeping 10ms between messages so the client can poll.
                let text = std::fs::read_to_string(&path).unwrap();
                for line in text.lines() {
                    if line.trim().is_empty() {
                        continue;
                    }
                    tx.send(Message::Text(line.to_string())).await.unwrap();
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                let _ = tx.close().await;
            });
        }
    });
    format!("ws://{}", addr)
}

#[tokio::test]
async fn soniox_byok_streams_tokens_through_to_events() {
    let url = spawn_replay_server(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/soniox_two_way_replay.jsonl"
    ))
    .await;
    let mut provider = SonioxBYOK::with_endpoint(&url);
    provider
        .open(SessionConfig {
            api_key: "test".into(),
            mode: Mode::Conversation,
            language_a: "en".into(),
            language_b: "vi".into(),
            capture_mic: false,
        })
        .await
        .unwrap();

    let mut finals = 0;
    let mut got_translation = false;
    let mut got_live = false;
    while let Some(ev) = provider.next_event().await {
        match ev {
            TranslationEvent::Finals { tokens, lag_ms } => {
                finals += tokens.len();
                if tokens.iter().any(|t| {
                    matches!(
                        t.status,
                        voxtide_core::translation::tokens::TranslationStatus::Translation
                    )
                }) {
                    got_translation = true;
                }
                // Both finals-bearing fixture frames carry a
                // final_audio_proc_ms watermark, so lag must be Some. No audio
                // was ever sent in this test, so the subtraction saturates to
                // 0 (the exact arithmetic is pinned by
                // finals_lag_is_audio_sent_minus_watermark below).
                assert_eq!(lag_ms, Some(0), "watermark frames must report a lag");
            }
            TranslationEvent::Live { .. } => {
                got_live = true;
            }
            TranslationEvent::Stopped => break,
            _ => {}
        }
    }
    assert!(got_live);
    assert!(got_translation);
    assert_eq!(finals, 4);
}

#[tokio::test]
async fn soniox_endpoint_marker_becomes_utterance_break() {
    let url = spawn_replay_server(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/soniox_endpoint_replay.jsonl"
    ))
    .await;
    let mut provider = SonioxBYOK::with_endpoint(&url);
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

    // Record the event order: the `<end>` token must surface as exactly one
    // UtteranceBreak, positioned between the two final utterances.
    let mut seq: Vec<&'static str> = Vec::new();
    while let Some(ev) = provider.next_event().await {
        match ev {
            TranslationEvent::Finals { tokens, .. } => {
                // Count per token so the assertion is independent of how the
                // provider batches finals into frames.
                seq.extend(tokens.iter().map(|_| "final"));
            }
            TranslationEvent::UtteranceBreak => seq.push("break"),
            TranslationEvent::Stopped => break,
            _ => {}
        }
    }
    assert_eq!(
        seq,
        vec!["final", "break", "final"],
        "endpoint <end> token should emit one UtteranceBreak between the two finals"
    );
}

/// Server that waits for the initial config AND `n_audio` binary audio frames
/// before replaying its canned frames. Receiving the N-th audio frame
/// happens-after the client's N-th successful WS send, so the client's
/// audio-sent watermark is fully settled (deterministic) by the time the
/// token frame arrives — no sleep-based racing.
async fn spawn_audio_gated_server(n_audio: usize, frames: &'static [&'static str]) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            let ws = accept_async(stream).await.unwrap();
            let (mut tx, mut rx) = ws.split();
            // 1) initial JSON config
            let _ = rx.next().await;
            // 2) the client's audio
            let mut seen = 0usize;
            while seen < n_audio {
                match rx.next().await {
                    Some(Ok(m)) if m.is_binary() => seen += 1,
                    Some(Ok(_)) => {}
                    _ => return,
                }
            }
            // 3) replay
            for f in frames {
                if tx.send(Message::Text((*f).to_string())).await.is_err() {
                    return;
                }
            }
            let _ = tx.close().await;
        }
    });
    format!("ws://{}", addr)
}

#[tokio::test]
async fn finals_lag_is_audio_sent_minus_watermark() {
    let url = spawn_audio_gated_server(
        8,
        &[
            r#"{"tokens":[{"text":"Hi","is_final":true,"language":"en","translation_status":"original"}],"final_audio_proc_ms":300}"#,
            r#"{"finished":true}"#,
        ],
    )
    .await;
    let mut provider = SonioxBYOK::with_endpoint(&url);
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

    // 8 × 3200 bytes of pcm_s16le @ 16 kHz mono = 8 × 100 ms = 800 ms of audio.
    for _ in 0..8 {
        provider.send_audio(vec![0u8; 3200]).await.unwrap();
    }

    let mut lag = None;
    while let Some(ev) = provider.next_event().await {
        match ev {
            TranslationEvent::Finals { lag_ms, .. } => lag = lag_ms,
            TranslationEvent::Stopped => break,
            _ => {}
        }
    }
    assert_eq!(
        lag,
        Some(500),
        "lag must be audio written to the socket (800 ms) minus the provider's \
         fully-processed watermark (300 ms)"
    );
    provider.close().await.unwrap();
}
