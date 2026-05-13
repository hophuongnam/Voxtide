use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSession {
    pub started_at: i64,
    pub mode: String,
    pub lang_a: String,
    pub lang_b: String,
    pub device_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SessionRow {
    pub id: i64,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub mode: String,
    pub lang_a: String,
    pub lang_b: String,
    pub device_label: Option<String>,
    pub duration_ms: Option<i64>,
}

pub struct Sessions;

impl Sessions {
    pub async fn create(pool: &SqlitePool, n: NewSession) -> Result<i64> {
        let result = sqlx::query(
            "INSERT INTO sessions(started_at, mode, lang_a, lang_b, device_label) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(n.started_at)
        .bind(&n.mode)
        .bind(&n.lang_a)
        .bind(&n.lang_b)
        .bind(&n.device_label)
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn finish(pool: &SqlitePool, id: i64, ended_at: i64) -> Result<()> {
        sqlx::query(
            "UPDATE sessions SET ended_at = ?1, duration_ms = ?1 - started_at WHERE id = ?2",
        )
        .bind(ended_at)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn list(pool: &SqlitePool, limit: i64) -> Result<Vec<SessionRow>> {
        let rows = sqlx::query_as::<_, SessionRow>(
            "SELECT id, started_at, ended_at, mode, lang_a, lang_b, device_label, duration_ms \
             FROM sessions ORDER BY started_at DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn get(pool: &SqlitePool, id: i64) -> Result<Option<SessionRow>> {
        let row = sqlx::query_as::<_, SessionRow>(
            "SELECT id, started_at, ended_at, mode, lang_a, lang_b, device_label, duration_ms \
             FROM sessions WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
        Ok(row)
    }
}
