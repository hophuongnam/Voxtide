use voxtide_core::persistence::sessions::{NewSession, SessionRow, Sessions};
use voxtide_core::persistence::Store;

async fn open_store() -> (Store, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("v.db")).await.unwrap();
    (store, dir)
}

#[tokio::test]
async fn create_session_returns_row_with_id() {
    let (s, _dir) = open_store().await;
    let id = Sessions::create(s.pool(), NewSession {
        started_at: 1_700_000_000_000,
        mode: "meeting".into(),
        lang_a: "en".into(),
        lang_b: "vi".into(),
        device_label: Some("Zoom Meeting".into()),
    }).await.unwrap();
    assert!(id > 0);
    let rows = Sessions::list(s.pool(), 50).await.unwrap();
    assert_eq!(rows.len(), 1);
    let r: &SessionRow = &rows[0];
    assert_eq!(r.mode, "meeting");
    assert_eq!(r.lang_a, "en");
    assert_eq!(r.device_label.as_deref(), Some("Zoom Meeting"));
}

#[tokio::test]
async fn finish_sets_ended_at_and_duration() {
    let (s, _dir) = open_store().await;
    let id = Sessions::create(s.pool(), NewSession {
        started_at: 1_700_000_000_000,
        mode: "conversation".into(),
        lang_a: "en".into(),
        lang_b: "ja".into(),
        device_label: None,
    }).await.unwrap();
    Sessions::finish(s.pool(), id, 1_700_000_001_500).await.unwrap();
    let rows = Sessions::list(s.pool(), 50).await.unwrap();
    let r = &rows[0];
    assert_eq!(r.ended_at, Some(1_700_000_001_500));
    assert_eq!(r.duration_ms, Some(1500));
}
