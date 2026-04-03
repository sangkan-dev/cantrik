PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS sessions (
    id TEXT NOT NULL PRIMARY KEY,
    project_fingerprint TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sessions_project_fingerprint
    ON sessions(project_fingerprint);

CREATE TABLE IF NOT EXISTS messages (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    approx_tokens INTEGER,
    ordinal INTEGER NOT NULL,
    UNIQUE(session_id, ordinal)
);

CREATE INDEX IF NOT EXISTS idx_messages_session_ordinal ON messages(session_id, ordinal);

CREATE TABLE IF NOT EXISTS session_summaries (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    text TEXT NOT NULL,
    covers_up_to_ordinal INTEGER NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_summaries_session ON session_summaries(session_id, id);

CREATE TABLE IF NOT EXISTS session_decisions (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    text TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS adaptive_stub (
    key TEXT NOT NULL PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
