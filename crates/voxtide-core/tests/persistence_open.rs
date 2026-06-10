use std::path::Path;
use std::str::FromStr;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;
use voxtide_core::persistence::Store;

#[tokio::test]
async fn opening_store_creates_tables() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("voxtide.db");
    let store = Store::open(&db).await.unwrap();
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type IN ('table','index') ORDER BY name",
    )
    .fetch_all(store.pool())
    .await
    .unwrap();
    let names: Vec<String> = rows.into_iter().map(|(n,)| n).collect();
    for required in ["sessions", "tokens", "idx_tokens_session", "tokens_fts"] {
        assert!(
            names.iter().any(|n| n == required),
            "missing {required}: have {names:?}"
        );
    }
}

/// Trigger-aware statement splitter mirroring the one in `persistence::mod`.
/// Used here only to lay down a *legacy* (pre-versioning) schema, exactly the
/// way the original `migrate()` did, so the test exercises the real upgrade path
/// rather than a hand-rolled approximation.
fn split_statements(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut in_trigger = false;
    for line in s.lines() {
        let t = line.trim();
        if t.to_uppercase().starts_with("CREATE TRIGGER") {
            in_trigger = true;
        }
        buf.push_str(line);
        buf.push('\n');
        if t.ends_with(';') {
            if in_trigger && !t.contains("END;") {
                continue;
            }
            out.push(std::mem::take(&mut buf));
            in_trigger = false;
        }
    }
    if !buf.trim().is_empty() {
        out.push(buf);
    }
    out
}

async fn raw_pool(path: &Path) -> SqlitePool {
    let opts = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))
        .unwrap()
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true);
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(opts)
        .await
        .unwrap()
}

/// Lay down a legacy DB: the 0001 schema applied with no `user_version` stamp
/// (the way installs predating the versioned runner look on disk), populated
/// with a session and a token carrying a *session-relative* `ts_ms`.
async fn build_legacy_db(path: &Path) {
    let schema = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/persistence/migrations/0001_init.sql"
    ))
    .unwrap();
    let pool = raw_pool(path).await;
    for stmt in split_statements(&schema) {
        sqlx::query(&stmt).execute(&pool).await.unwrap();
    }
    // A finalized session (ended_at set so reconcile_stale leaves it alone) with
    // a wall-clock start; the token's ts_ms is stored session-relative (65s in),
    // mimicking the pre-fix persistence path.
    sqlx::query(
        "INSERT INTO sessions(id, started_at, ended_at, mode, lang_a, lang_b, duration_ms) \
         VALUES (1, 1750000000000, 1750000300000, 'meeting', 'en', 'vi', 300000)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO tokens(id, session_id, ts_ms, text, status) \
         VALUES (1, 1, 65000, 'Hello', 'original')",
    )
    .execute(&pool)
    .await
    .unwrap();
    // Confirm we really built a pre-versioning DB.
    let v: i64 = sqlx::query_scalar("PRAGMA user_version")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(v, 0, "fixture must start unversioned");
    pool.close().await;
}

async fn user_version(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar("PRAGMA user_version")
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn has_is_break_column(pool: &SqlitePool) -> bool {
    let cols: Vec<(i64, String)> = sqlx::query_as("PRAGMA table_info(tokens)")
        .fetch_all(pool)
        .await
        .unwrap()
        .into_iter()
        .map(|r: (i64, String, String, i64, Option<String>, i64)| (r.0, r.1))
        .collect();
    cols.iter().any(|(_, name)| name == "is_break")
}

#[tokio::test]
async fn legacy_db_migrates_relative_ts_to_epoch_and_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("voxtide.db");
    build_legacy_db(&db).await;

    // First open runs the 0002 migration: stamps v1, applies 0002 → v2.
    let store = Store::open(&db).await.unwrap();
    let ts: i64 = sqlx::query_scalar("SELECT ts_ms FROM tokens WHERE id = 1")
        .fetch_one(store.pool())
        .await
        .unwrap();
    assert_eq!(
        ts, 1_750_000_065_000,
        "relative ts (65000) must be shifted to epoch by adding started_at"
    );
    assert_eq!(user_version(store.pool()).await, 2, "must end at version 2");
    assert!(
        has_is_break_column(store.pool()).await,
        "0002 must add the is_break column"
    );
    drop(store);

    // Second open is a no-op for the data: the version gate prevents re-adding
    // started_at (which would otherwise double the epoch).
    let store2 = Store::open(&db).await.unwrap();
    let ts2: i64 = sqlx::query_scalar("SELECT ts_ms FROM tokens WHERE id = 1")
        .fetch_one(store2.pool())
        .await
        .unwrap();
    assert_eq!(ts2, 1_750_000_065_000, "second open must not re-shift ts");
    assert_eq!(user_version(store2.pool()).await, 2);
}

#[tokio::test]
async fn fresh_store_open_ends_at_version_2_with_is_break() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("voxtide.db");
    let store = Store::open(&db).await.unwrap();
    assert_eq!(
        user_version(store.pool()).await,
        2,
        "a fresh DB must be created at the latest schema version"
    );
    assert!(
        has_is_break_column(store.pool()).await,
        "fresh DB must have the is_break column"
    );
}
