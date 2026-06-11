use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::persistence::sessions::SessionRow;
use crate::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewToken {
    pub session_id: i64,
    pub ts_ms: i64,
    pub text: String,
    pub language: Option<String>,
    pub status: String,
    pub speaker: Option<String>,
    /// 1 = utterance-break marker row (empty text, breaks BOTH columns on
    /// replay), 0 = ordinary token. i64 (not bool) so the wire shape matches
    /// the SQLite column and the TS `is_break: number` type.
    pub is_break: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TokenRow {
    pub id: i64,
    pub session_id: i64,
    pub ts_ms: i64,
    pub text: String,
    pub language: Option<String>,
    pub status: String,
    pub speaker: Option<String>,
    /// See [`NewToken::is_break`].
    pub is_break: i64,
}

pub struct Tokens;

impl Tokens {
    pub async fn insert(pool: &SqlitePool, t: NewToken) -> Result<i64> {
        // Use execute() + last_insert_rowid() (Task 17 found that RETURNING+pool can race).
        let res = sqlx::query(
            "INSERT INTO tokens(session_id, ts_ms, text, language, status, speaker, is_break) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(t.session_id)
        .bind(t.ts_ms)
        .bind(&t.text)
        .bind(&t.language)
        .bind(&t.status)
        .bind(&t.speaker)
        .bind(t.is_break)
        .execute(pool)
        .await?;
        Ok(res.last_insert_rowid())
    }

    /// Insert a whole finals frame in ONE transaction: one commit (and at
    /// most one fsync) instead of N serial autocommits. The FTS triggers run
    /// inside the same transaction. Empty batches are a no-op.
    pub async fn insert_many(pool: &SqlitePool, batch: &[NewToken]) -> Result<()> {
        if batch.is_empty() {
            return Ok(());
        }
        let mut tx = pool.begin().await?;
        for t in batch {
            sqlx::query(
                "INSERT INTO tokens(session_id, ts_ms, text, language, status, speaker, is_break) \
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(t.session_id)
            .bind(t.ts_ms)
            .bind(&t.text)
            .bind(&t.language)
            .bind(&t.status)
            .bind(&t.speaker)
            .bind(t.is_break)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn list_by_session(pool: &SqlitePool, session_id: i64) -> Result<Vec<TokenRow>> {
        // Secondary sort by id (autoincrement = insertion order). Without it,
        // tokens sharing a ts_ms have indeterminate order — past-session view
        // could flip speaker A↔B chips between re-queries, since chip
        // assignment in the frontend uses first-seen order.
        let rows = sqlx::query_as::<_, TokenRow>(
            "SELECT id, session_id, ts_ms, text, language, status, speaker, is_break \
             FROM tokens WHERE session_id = ? ORDER BY ts_ms ASC, id ASC",
        )
        .bind(session_id)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    /// Full-text search returning the matching SESSIONS, newest first. The
    /// frontend previously mapped token hits onto its in-memory sidebar cache
    /// of recent sessions, silently dropping any match in an older session —
    /// returning the rows themselves makes every match reachable.
    pub async fn search_sessions(
        pool: &SqlitePool,
        query: &str,
        limit: i64,
    ) -> Result<Vec<SessionRow>> {
        let q = sanitize_fts(query);
        if q.is_empty() {
            return Ok(Vec::new());
        }
        let rows = sqlx::query_as::<_, SessionRow>(
            "SELECT s.id, s.started_at, s.ended_at, s.mode, s.lang_a, s.lang_b, \
                    s.device_label, s.duration_ms \
             FROM sessions s \
             WHERE s.id IN ( \
                 SELECT DISTINCT t.session_id FROM tokens_fts f \
                 JOIN tokens t ON t.id = f.rowid \
                 WHERE tokens_fts MATCH ? \
             ) \
             ORDER BY s.started_at DESC LIMIT ?",
        )
        .bind(q)
        .bind(limit)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}

fn sanitize_fts(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    trimmed
        .split_whitespace()
        .map(|w| format!("\"{}\"", w.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" ")
}
