PRAGMA foreign_keys = ON;

-- Sprint 19 — Adaptive Begawan (PRD §4.15): user approval / rejection summaries per project fingerprint.
CREATE TABLE IF NOT EXISTS approval_memory (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    project_fingerprint TEXT NOT NULL,
    tool_id TEXT NOT NULL,
    approved INTEGER NOT NULL,
    summary TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_approval_memory_project_id
    ON approval_memory(project_fingerprint, id DESC);
