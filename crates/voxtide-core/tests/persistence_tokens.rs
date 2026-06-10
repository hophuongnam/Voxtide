use voxtide_core::persistence::sessions::{NewSession, Sessions};
use voxtide_core::persistence::tokens::{NewToken, SearchHit, Tokens};
use voxtide_core::persistence::Store;

async fn open_store() -> Store {
    let dir = tempfile::tempdir().unwrap();
    Store::open(&dir.path().join("v.db")).await.unwrap()
}

#[tokio::test]
async fn insert_token_and_search_finds_match() {
    let s = open_store().await;
    let session_id = Sessions::create(
        s.pool(),
        NewSession {
            started_at: 0,
            mode: "meeting".into(),
            lang_a: "en".into(),
            lang_b: "vi".into(),
            device_label: None,
        },
    )
    .await
    .unwrap();

    Tokens::insert(
        s.pool(),
        NewToken {
            session_id,
            ts_ms: 100,
            text: "Hello world".into(),
            language: Some("en".into()),
            status: "original".into(),
            speaker: Some("1".into()),
            is_break: 0,
        },
    )
    .await
    .unwrap();
    Tokens::insert(
        s.pool(),
        NewToken {
            session_id,
            ts_ms: 200,
            text: "Xin chào thế giới".into(),
            language: Some("vi".into()),
            status: "translation".into(),
            speaker: Some("1".into()),
            is_break: 0,
        },
    )
    .await
    .unwrap();

    let hits: Vec<SearchHit> = Tokens::search(s.pool(), "world", 10).await.unwrap();
    assert_eq!(hits.len(), 1);
    assert!(hits[0].text.contains("Hello"));

    let by_session = Tokens::list_by_session(s.pool(), session_id).await.unwrap();
    assert_eq!(by_session.len(), 2);
}

#[tokio::test]
async fn deleting_session_cascades_to_tokens_and_fts() {
    let s = open_store().await;
    let session_id = Sessions::create(
        s.pool(),
        NewSession {
            started_at: 0,
            mode: "meeting".into(),
            lang_a: "en".into(),
            lang_b: "vi".into(),
            device_label: None,
        },
    )
    .await
    .unwrap();
    Tokens::insert(
        s.pool(),
        NewToken {
            session_id,
            ts_ms: 0,
            text: "ephemeral".into(),
            language: None,
            status: "original".into(),
            speaker: None,
            is_break: 0,
        },
    )
    .await
    .unwrap();
    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(session_id)
        .execute(s.pool())
        .await
        .unwrap();
    let hits = Tokens::search(s.pool(), "ephemeral", 10).await.unwrap();
    assert!(
        hits.is_empty(),
        "FTS row should have been removed by trigger"
    );
}

#[tokio::test]
async fn insert_many_lands_the_whole_batch() {
    let s = open_store().await;
    let session_id = Sessions::create(
        s.pool(),
        NewSession {
            started_at: 0,
            mode: "meeting".into(),
            lang_a: "en".into(),
            lang_b: "vi".into(),
            device_label: None,
        },
    )
    .await
    .unwrap();

    let batch = vec![
        NewToken {
            session_id,
            ts_ms: 100,
            text: "one".into(),
            language: Some("en".into()),
            status: "original".into(),
            speaker: Some("A".into()),
            is_break: 0,
        },
        NewToken {
            session_id,
            ts_ms: 110,
            text: "hai".into(),
            language: Some("vi".into()),
            status: "translation".into(),
            speaker: Some("A".into()),
            is_break: 0,
        },
        NewToken {
            session_id,
            ts_ms: 120,
            text: "three".into(),
            language: Some("en".into()),
            status: "original".into(),
            speaker: Some("A".into()),
            is_break: 0,
        },
    ];
    Tokens::insert_many(s.pool(), &batch).await.unwrap();

    let rows = Tokens::list_by_session(s.pool(), session_id).await.unwrap();
    assert_eq!(rows.len(), 3, "every row of the batch must land");
    assert_eq!(
        rows.iter().map(|r| r.text.as_str()).collect::<Vec<_>>(),
        vec!["one", "hai", "three"]
    );
    // FTS triggers fire inside the transaction too.
    let hits = Tokens::search(s.pool(), "three", 10).await.unwrap();
    assert_eq!(hits.len(), 1);

    // Empty batch is a no-op, not an error.
    Tokens::insert_many(s.pool(), &[]).await.unwrap();
    let rows = Tokens::list_by_session(s.pool(), session_id).await.unwrap();
    assert_eq!(rows.len(), 3);
}

#[tokio::test]
async fn break_rows_round_trip_in_order() {
    let s = open_store().await;
    let session_id = Sessions::create(
        s.pool(),
        NewSession {
            started_at: 0,
            mode: "meeting".into(),
            lang_a: "en".into(),
            lang_b: "vi".into(),
            device_label: None,
        },
    )
    .await
    .unwrap();

    for t in [
        NewToken {
            session_id,
            ts_ms: 100,
            text: "before".into(),
            language: Some("en".into()),
            status: "original".into(),
            speaker: Some("1".into()),
            is_break: 0,
        },
        NewToken {
            session_id,
            ts_ms: 150,
            text: String::new(),
            language: None,
            status: "none".into(),
            speaker: None,
            is_break: 1,
        },
        NewToken {
            session_id,
            ts_ms: 200,
            text: "after".into(),
            language: Some("en".into()),
            status: "original".into(),
            speaker: Some("1".into()),
            is_break: 0,
        },
    ] {
        Tokens::insert(s.pool(), t).await.unwrap();
    }

    let rows = Tokens::list_by_session(s.pool(), session_id).await.unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(
        rows.iter().map(|r| r.is_break).collect::<Vec<_>>(),
        vec![0, 1, 0],
        "is_break flags must survive the round trip in token order"
    );
    assert_eq!(rows[1].text, "", "break rows carry no text");
}
