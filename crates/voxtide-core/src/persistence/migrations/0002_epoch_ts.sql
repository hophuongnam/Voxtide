-- Adds the pause-break marker column (consumed by the pause-persistence work)
-- and rewrites stored token timestamps from session-relative offsets to
-- wall-clock epoch ms.
--
-- Pre-fix, finals were persisted as `ts_ms - started_at`, so reopened sessions
-- rendered every row at the 1970 epoch (the frontend reads ts_ms as absolute
-- ms). This shifts each relative value back to epoch by adding the owning
-- session's started_at. The `< 100000000000` guard (relative values are < ~3
-- years in ms; real epoch values are ~1.78e12) means already-epoch rows are
-- never touched, and the migration's user_version gate ensures it runs at most
-- once regardless. The EXISTS guard keeps the NOT NULL ts_ms intact even if a
-- token's session row is somehow missing (the subquery would otherwise yield
-- NULL and the UPDATE would fail).
ALTER TABLE tokens ADD COLUMN is_break INTEGER NOT NULL DEFAULT 0;

UPDATE tokens
SET ts_ms = ts_ms + (SELECT started_at FROM sessions s WHERE s.id = tokens.session_id)
WHERE ts_ms < 100000000000
  AND EXISTS (SELECT 1 FROM sessions s WHERE s.id = tokens.session_id);
