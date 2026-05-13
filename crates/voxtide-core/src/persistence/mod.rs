use std::path::Path;
use std::str::FromStr;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;

use crate::Result;

pub mod sessions;
pub mod tokens;

pub struct Store {
    pool: SqlitePool,
}

impl Store {
    pub async fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let opts = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new().max_connections(8).connect_with(opts).await?;
        let store = Self { pool };
        store.migrate().await?;
        Ok(store)
    }

    pub fn pool(&self) -> &SqlitePool { &self.pool }

    async fn migrate(&self) -> Result<()> {
        let sql = include_str!("migrations/0001_init.sql");
        for stmt in split_statements(sql) {
            sqlx::query(&stmt).execute(&self.pool).await?;
        }
        Ok(())
    }
}

fn split_statements(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut in_trigger = false;
    for line in s.lines() {
        let t = line.trim();
        if t.to_uppercase().starts_with("CREATE TRIGGER") { in_trigger = true; }
        buf.push_str(line); buf.push('\n');
        if t.ends_with(';') {
            if in_trigger && !t.contains("END;") { continue; }
            out.push(std::mem::take(&mut buf));
            in_trigger = false;
        }
    }
    if !buf.trim().is_empty() { out.push(buf); }
    out
}
