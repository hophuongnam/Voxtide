use voxtide_core::persistence::sessions::{NewSession, Sessions};
use voxtide_core::persistence::tokens::{NewToken, Tokens};
use voxtide_core::persistence::Store;

/// Returns the `TempDir` guard alongside the store: dropping it deletes the
/// directory while the pool still points into it, and any LATER lazily-opened
/// pool connection (or SQLite journal file) then fails with "unable to open
/// database file" — a parallelism-dependent flake, not a persistence bug.
/// Same contract as `persistence_sessions.rs`'s helper.
async fn open_store() -> (Store, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("v.db")).await.unwrap();
    (store, dir)
}

#[tokio::test]
async fn insert_token_and_search_finds_match() {
    let (s, _dir) = open_store().await;
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

    let hits = Tokens::search_sessions(s.pool(), "world", 10)
        .await
        .unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, session_id);

    let by_session = Tokens::list_by_session(s.pool(), session_id).await.unwrap();
    assert_eq!(by_session.len(), 2);
}

#[tokio::test]
async fn deleting_session_cascades_to_tokens_and_fts() {
    let (s, _dir) = open_store().await;
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
    let hits = Tokens::search_sessions(s.pool(), "ephemeral", 10)
        .await
        .unwrap();
    assert!(
        hits.is_empty(),
        "FTS row should have been removed by trigger"
    );
}

#[tokio::test]
async fn search_sessions_finds_rows_beyond_any_cache_and_dedupes() {
    let (s, _dir) = open_store().await;
    let mk = |started_at: i64| NewSession {
        started_at,
        mode: "meeting".into(),
        lang_a: "en".into(),
        lang_b: "vi".into(),
        device_label: None,
    };
    // The OLDER session holds the match — a sidebar cache of recent rows
    // would never contain it; search must return the row itself.
    let old_id = Sessions::create(s.pool(), mk(1_000)).await.unwrap();
    let new_id = Sessions::create(s.pool(), mk(2_000)).await.unwrap();
    let tok = |session_id: i64, ts_ms: i64, text: &str| NewToken {
        session_id,
        ts_ms,
        text: text.into(),
        language: Some("en".into()),
        status: "original".into(),
        speaker: None,
        is_break: 0,
    };
    Tokens::insert(s.pool(), tok(old_id, 10, "alpha particle"))
        .await
        .unwrap();
    Tokens::insert(s.pool(), tok(old_id, 20, "alpha again"))
        .await
        .unwrap();
    Tokens::insert(s.pool(), tok(new_id, 30, "gamma ray"))
        .await
        .unwrap();

    let rows = Tokens::search_sessions(s.pool(), "alpha", 50)
        .await
        .unwrap();
    assert_eq!(
        rows.iter().map(|r| r.id).collect::<Vec<_>>(),
        vec![old_id],
        "one SessionRow per matching session (DISTINCT), even with two hits"
    );

    // Both sessions match → recency order (newest first).
    Tokens::insert(s.pool(), tok(new_id, 40, "alpha too"))
        .await
        .unwrap();
    let rows = Tokens::search_sessions(s.pool(), "alpha", 50)
        .await
        .unwrap();
    assert_eq!(
        rows.iter().map(|r| r.id).collect::<Vec<_>>(),
        vec![new_id, old_id]
    );
}

#[tokio::test]
async fn insert_many_lands_the_whole_batch() {
    let (s, _dir) = open_store().await;
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
    let hits = Tokens::search_sessions(s.pool(), "three", 10)
        .await
        .unwrap();
    assert_eq!(hits.len(), 1);

    // Empty batch is a no-op, not an error.
    Tokens::insert_many(s.pool(), &[]).await.unwrap();
    let rows = Tokens::list_by_session(s.pool(), session_id).await.unwrap();
    assert_eq!(rows.len(), 3);
}

#[tokio::test]
async fn break_rows_round_trip_in_order() {
    let (s, _dir) = open_store().await;
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
