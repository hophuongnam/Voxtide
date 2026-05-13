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
    let session_id = Sessions::create(s.pool(), NewSession {
        started_at: 0, mode: "meeting".into(),
        lang_a: "en".into(), lang_b: "vi".into(), device_label: None,
    }).await.unwrap();

    Tokens::insert(s.pool(), NewToken {
        session_id, ts_ms: 100, text: "Hello world".into(),
        language: Some("en".into()), status: "original".into(), speaker: Some("1".into()),
    }).await.unwrap();
    Tokens::insert(s.pool(), NewToken {
        session_id, ts_ms: 200, text: "Xin chào thế giới".into(),
        language: Some("vi".into()), status: "translation".into(), speaker: Some("1".into()),
    }).await.unwrap();

    let hits: Vec<SearchHit> = Tokens::search(s.pool(), "world", 10).await.unwrap();
    assert_eq!(hits.len(), 1);
    assert!(hits[0].text.contains("Hello"));

    let by_session = Tokens::list_by_session(s.pool(), session_id).await.unwrap();
    assert_eq!(by_session.len(), 2);
}

#[tokio::test]
async fn deleting_session_cascades_to_tokens_and_fts() {
    let s = open_store().await;
    let session_id = Sessions::create(s.pool(), NewSession {
        started_at: 0, mode: "meeting".into(),
        lang_a: "en".into(), lang_b: "vi".into(), device_label: None,
    }).await.unwrap();
    Tokens::insert(s.pool(), NewToken {
        session_id, ts_ms: 0, text: "ephemeral".into(),
        language: None, status: "original".into(), speaker: None,
    }).await.unwrap();
    sqlx::query("DELETE FROM sessions WHERE id = ?").bind(session_id).execute(s.pool()).await.unwrap();
    let hits = Tokens::search(s.pool(), "ephemeral", 10).await.unwrap();
    assert!(hits.is_empty(), "FTS row should have been removed by trigger");
}
