use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewToken {
    pub session_id: i64,
    pub ts_ms: i64,
    pub text: String,
    pub language: Option<String>,
    pub status: String,
    pub speaker: Option<String>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SearchHit {
    pub id: i64,
    pub session_id: i64,
    pub ts_ms: i64,
    pub text: String,
}

pub struct Tokens;

impl Tokens {
    pub async fn insert(pool: &SqlitePool, t: NewToken) -> Result<i64> {
        // Use execute() + last_insert_rowid() (Task 17 found that RETURNING+pool can race).
        let res = sqlx::query(
            "INSERT INTO tokens(session_id, ts_ms, text, language, status, speaker) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(t.session_id)
        .bind(t.ts_ms)
        .bind(&t.text)
        .bind(&t.language)
        .bind(&t.status)
        .bind(&t.speaker)
        .execute(pool)
        .await?;
        Ok(res.last_insert_rowid())
    }

    pub async fn list_by_session(pool: &SqlitePool, session_id: i64) -> Result<Vec<TokenRow>> {
        let rows = sqlx::query_as::<_, TokenRow>(
            "SELECT id, session_id, ts_ms, text, language, status, speaker \
             FROM tokens WHERE session_id = ? ORDER BY ts_ms ASC",
        )
        .bind(session_id)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn search(pool: &SqlitePool, query: &str, limit: i64) -> Result<Vec<SearchHit>> {
        let q = sanitize_fts(query);
        if q.is_empty() {
            return Ok(Vec::new());
        }
        let rows = sqlx::query_as::<_, SearchHit>(
            "SELECT t.id, t.session_id, t.ts_ms, t.text \
             FROM tokens_fts f JOIN tokens t ON t.id = f.rowid \
             WHERE tokens_fts MATCH ? ORDER BY t.ts_ms DESC LIMIT ?",
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
