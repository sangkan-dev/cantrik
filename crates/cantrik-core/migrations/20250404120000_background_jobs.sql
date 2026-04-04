PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS background_jobs (
    id TEXT NOT NULL PRIMARY KEY,
    project_fingerprint TEXT NOT NULL,
    cwd TEXT NOT NULL,
    goal TEXT NOT NULL,
    state TEXT NOT NULL,
    last_error TEXT,
    approval_hint TEXT,
    rounds_done INTEGER NOT NULL DEFAULT 0,
    notify_on_approval INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    heartbeat_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_background_jobs_project
    ON background_jobs(project_fingerprint);

CREATE INDEX IF NOT EXISTS idx_background_jobs_state_created
    ON background_jobs(state, created_at);
