use std::path::Path;
use std::str::FromStr;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
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
            // The documented safe pairing with WAL: commits skip the per-txn
            // fsync (the WAL is synced at checkpoints), so the token hot path
            // isn't serialized on disk flushes. An OS crash/power loss can
            // lose the most recent commits but never corrupts the database;
            // sqlx 0.7 otherwise leaves SQLite's default of FULL in place.
            .synchronous(SqliteSynchronous::Normal)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .connect_with(opts)
            .await?;
        let store = Self { pool };
        store.migrate().await?;
        // Purge Soniox control markers ('<end>', '<fin>', …) persisted as
        // ordinary tokens by pre-v0.1.6 builds. New writes never store them
        // (the provider strips markers on the wire), so this only ever
        // touches legacy rows; the tokens_ad FTS trigger de-indexes each
        // delete. GLOB '<*>' is exactly the frontend's replay filter
        // (starts '<' AND ends '>'); break rows are excluded because their
        // empty text can never match.
        sqlx::query("DELETE FROM tokens WHERE is_break = 0 AND text GLOB '<*>'")
            .execute(&store.pool)
            .await?;
        // Repair orphan sessions left `ended_at IS NULL` by a prior
        // kill/crash/quit-while-recording. Runs before the controller exists,
        // so every NULL-ended row here is definitively stale.
        sessions::Sessions::reconcile_stale(&store.pool).await?;
        Ok(store)
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Apply any migrations newer than the DB's `user_version`, in order.
    ///
    /// Each migration's SQL is executed through the same trigger-aware
    /// [`split_statements`] path the initial schema always used (0001 defines
    /// FTS5 triggers with `BEGIN … END;` blocks that must not be split on their
    /// internal `;`).
    ///
    /// Atomicity: every migration runs in ONE transaction together with its
    /// `PRAGMA user_version` bump. Both DDL (ALTER/CREATE) and `user_version`
    /// are transactional in SQLite, so the statements and the version stamp
    /// commit (or roll back) as a unit. This is what makes a power-loss
    /// mid-migration safe: if the process dies before `COMMIT`, the partial
    /// schema change is discarded and the version is unchanged, so the next
    /// `open()` re-runs the migration cleanly. (Without the transaction, 0002's
    /// `ALTER … ADD COLUMN is_break` could auto-commit while the version bump
    /// did not, and the re-run would fail with "duplicate column".) The 0002
    /// timestamp rewrite is additionally idempotent on its own — its
    /// `ts_ms < 1e11` predicate skips rows already on the epoch clock.
    async fn migrate(&self) -> Result<()> {
        let mut v: i64 = sqlx::query_scalar("PRAGMA user_version")
            .fetch_one(&self.pool)
            .await?;
        // Installs predating the versioned runner have the 0001 tables but
        // `user_version == 0`. Stamp them as v1 so the loop applies only the
        // genuinely-newer migrations (and never re-runs 0001's `IF NOT EXISTS`
        // schema, which is harmless anyway but pointless here). A truly fresh DB
        // has no `tokens` table, so it stays at 0 and the loop creates the
        // schema from 0001 onward.
        if v == 0 {
            let has_tokens: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='tokens'",
            )
            .fetch_one(&self.pool)
            .await?;
            if has_tokens > 0 {
                sqlx::query("PRAGMA user_version = 1")
                    .execute(&self.pool)
                    .await?;
                v = 1;
            }
        }
        for (ver, sql) in MIGRATIONS {
            if *ver > v {
                let mut tx = self.pool.begin().await?;
                for stmt in split_statements(sql) {
                    sqlx::query(&stmt).execute(&mut *tx).await?;
                }
                // PRAGMA user_version does not accept a bound parameter, so the
                // version literal is interpolated. It originates from the
                // hard-coded MIGRATIONS table — never user input.
                sqlx::query(&format!("PRAGMA user_version = {ver}"))
                    .execute(&mut *tx)
                    .await?;
                tx.commit().await?;
            }
        }
        Ok(())
    }
}

/// Schema migrations applied in ascending order, gated by `PRAGMA user_version`.
/// Append-only: never edit a shipped migration's SQL (it may already be applied
/// in the field) — add a new numbered entry instead.
const MIGRATIONS: &[(i64, &str)] = &[
    (1, include_str!("migrations/0001_init.sql")),
    (2, include_str!("migrations/0002_epoch_ts.sql")),
];

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
