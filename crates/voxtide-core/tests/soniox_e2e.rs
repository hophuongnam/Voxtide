use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use voxtide_core::translation::soniox::SonioxBYOK;
use voxtide_core::translation::{
    Mode, SessionConfig, TranslationEvent, TranslationProvider,
};

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
        })
        .await
        .unwrap();

    let mut finals = 0;
    let mut got_translation = false;
    let mut got_live = false;
    while let Some(ev) = provider.next_event().await {
        match ev {
            TranslationEvent::Final { status, .. } => {
                finals += 1;
                if matches!(
                    status,
                    voxtide_core::translation::tokens::TranslationStatus::Translation
                ) {
                    got_translation = true;
                }
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
        })
        .await
        .unwrap();

    // Record the event order: the `<end>` token must surface as exactly one
    // UtteranceBreak, positioned between the two final utterances.
    let mut seq: Vec<&'static str> = Vec::new();
    while let Some(ev) = provider.next_event().await {
        match ev {
            TranslationEvent::Final { .. } => seq.push("final"),
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
