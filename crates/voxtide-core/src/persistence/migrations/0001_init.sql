CREATE TABLE sessions (
  id            INTEGER PRIMARY KEY,
  started_at    INTEGER NOT NULL,
  ended_at      INTEGER,
  mode          TEXT NOT NULL,
  lang_a        TEXT NOT NULL,
  lang_b        TEXT NOT NULL,
  device_label  TEXT,
  duration_ms   INTEGER
);

CREATE TABLE tokens (
  id          INTEGER PRIMARY KEY,
  session_id  INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  ts_ms       INTEGER NOT NULL,
  text        TEXT NOT NULL,
  language    TEXT,
  status      TEXT NOT NULL,
  speaker     TEXT
);

CREATE INDEX idx_tokens_session ON tokens(session_id, ts_ms);

CREATE VIRTUAL TABLE tokens_fts USING fts5(
  text,
  content='tokens',
  content_rowid='id'
);

CREATE TRIGGER tokens_ai AFTER INSERT ON tokens BEGIN
  INSERT INTO tokens_fts(rowid, text) VALUES (new.id, new.text);
END;
CREATE TRIGGER tokens_ad AFTER DELETE ON tokens BEGIN
  INSERT INTO tokens_fts(tokens_fts, rowid, text) VALUES('delete', old.id, old.text);
END;
