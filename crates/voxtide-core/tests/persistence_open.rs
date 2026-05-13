use voxtide_core::persistence::Store;

#[tokio::test]
async fn opening_store_creates_tables() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("voxtide.db");
    let store = Store::open(&db).await.unwrap();
    let rows: Vec<(String,)> = sqlx::query_as("SELECT name FROM sqlite_master WHERE type IN ('table','index') ORDER BY name")
        .fetch_all(store.pool())
        .await.unwrap();
    let names: Vec<String> = rows.into_iter().map(|(n,)| n).collect();
    for required in ["sessions", "tokens", "idx_tokens_session", "tokens_fts"] {
        assert!(names.iter().any(|n| n == required), "missing {required}: have {names:?}");
    }
}
