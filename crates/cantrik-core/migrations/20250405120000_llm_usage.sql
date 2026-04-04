CREATE TABLE IF NOT EXISTS llm_usage (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    session_id TEXT REFERENCES sessions(id) ON DELETE SET NULL,
    project_fingerprint TEXT NOT NULL,
    at TEXT NOT NULL,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    tier TEXT,
    input_chars INTEGER NOT NULL,
    output_chars INTEGER NOT NULL,
    cost_usd_approx REAL NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_llm_usage_fp_session
    ON llm_usage(project_fingerprint, session_id);

CREATE INDEX IF NOT EXISTS idx_llm_usage_fp_at
    ON llm_usage(project_fingerprint, at);
