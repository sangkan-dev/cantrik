//! Tier 2 session memory: SQLite history, summaries, decisions (Sprint 7).

mod anchors;
mod context;
mod db;
mod paths;

pub use anchors::load_anchors_combined;
pub use context::{build_llm_prompt, maybe_summarize_session};
pub use db::connect_pool;
pub use paths::{
    ENV_MEMORY_DB, global_anchors_path, memory_db_path, project_anchors_path, share_dir,
};

use std::path::Path;

use sha2::{Digest, Sha256};
use sqlx::Row;
pub use sqlx::SqlitePool;
use thiserror::Error;

pub const ENV_NO_SUMMARY: &str = "CANTRIK_NO_SUMMARY";

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("sql: {0}")]
    Sql(#[from] sqlx::Error),
    #[error("migrate: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("llm: {0}")]
    Llm(String),
}

/// Stable per-project key from canonical working directory.
pub fn project_fingerprint(cwd: &Path) -> String {
    let canon = std::fs::canonicalize(cwd).unwrap_or_else(|_| cwd.to_path_buf());
    let s = canon.to_string_lossy();
    hex::encode(Sha256::digest(s.as_bytes()))
}

pub async fn open_or_create_session(pool: &SqlitePool, cwd: &Path) -> Result<String, SessionError> {
    let fp = project_fingerprint(cwd);
    let row = sqlx::query(
        "SELECT id FROM sessions WHERE project_fingerprint = ? ORDER BY updated_at DESC LIMIT 1",
    )
    .bind(&fp)
    .fetch_optional(pool)
    .await?;
    if let Some(row) = row {
        return Ok(row.get::<String, _>(0));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO sessions (id, project_fingerprint, created_at, updated_at) VALUES (?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&fp)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(id)
}

pub async fn touch_session(pool: &SqlitePool, session_id: &str) -> Result<(), SessionError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("UPDATE sessions SET updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(session_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn append_message(
    pool: &SqlitePool,
    session_id: &str,
    role: &str,
    content: &str,
) -> Result<(), SessionError> {
    let ord: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(ordinal), 0) + 1 FROM messages WHERE session_id = ?",
    )
    .bind(session_id)
    .fetch_one(pool)
    .await?;

    let now = chrono::Utc::now().to_rfc3339();
    let approx = (content.len() / 4).max(1) as i32;
    sqlx::query(
        "INSERT INTO messages (session_id, role, content, created_at, approx_tokens, ordinal) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(session_id)
    .bind(role)
    .bind(content)
    .bind(&now)
    .bind(approx)
    .bind(ord)
    .execute(pool)
    .await?;
    touch_session(pool, session_id).await?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct MessageEntry {
    pub ordinal: i64,
    pub role: String,
    pub content: String,
}

pub async fn list_messages_after(
    pool: &SqlitePool,
    session_id: &str,
    after_ordinal: i64,
) -> Result<Vec<MessageEntry>, SessionError> {
    let rows = sqlx::query(
        "SELECT ordinal, role, content FROM messages WHERE session_id = ? AND ordinal > ? ORDER BY ordinal ASC",
    )
    .bind(session_id)
    .bind(after_ordinal)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| MessageEntry {
            ordinal: row.get(0),
            role: row.get(1),
            content: row.get(2),
        })
        .collect())
}

pub async fn list_all_messages_ordered(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Vec<MessageEntry>, SessionError> {
    let rows = sqlx::query(
        "SELECT ordinal, role, content FROM messages WHERE session_id = ? ORDER BY ordinal ASC",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| MessageEntry {
            ordinal: row.get(0),
            role: row.get(1),
            content: row.get(2),
        })
        .collect())
}

#[derive(Debug, Clone)]
pub struct SummaryHead {
    pub text: String,
    pub covers_up_to_ordinal: i64,
}

pub async fn latest_summary(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Option<SummaryHead>, SessionError> {
    let row = sqlx::query(
        "SELECT text, covers_up_to_ordinal FROM session_summaries WHERE session_id = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| SummaryHead {
        text: row.get(0),
        covers_up_to_ordinal: row.get(1),
    }))
}

pub async fn save_summary(
    pool: &SqlitePool,
    session_id: &str,
    text: &str,
    covers_up_to_ordinal: i64,
) -> Result<(), SessionError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO session_summaries (session_id, text, covers_up_to_ordinal, created_at) VALUES (?, ?, ?, ?)",
    )
    .bind(session_id)
    .bind(text)
    .bind(covers_up_to_ordinal)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn save_decision(
    pool: &SqlitePool,
    session_id: &str,
    text: &str,
) -> Result<(), SessionError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO session_decisions (session_id, text, created_at) VALUES (?, ?, ?)")
        .bind(session_id)
        .bind(text)
        .bind(&now)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn message_count(pool: &SqlitePool, session_id: &str) -> Result<i64, SessionError> {
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM messages WHERE session_id = ?")
        .bind(session_id)
        .fetch_one(pool)
        .await?;
    Ok(n)
}

#[derive(Debug, Clone)]
pub struct SessionListEntry {
    pub id: String,
    pub updated_at: String,
    pub message_count: i64,
}

pub async fn list_sessions_for_project(
    pool: &SqlitePool,
    cwd: &Path,
) -> Result<Vec<SessionListEntry>, SessionError> {
    let fp = project_fingerprint(cwd);
    let rows = sqlx::query(
        r#"SELECT s.id, s.updated_at,
           (SELECT COUNT(*) FROM messages m WHERE m.session_id = s.id) AS mc
           FROM sessions s WHERE s.project_fingerprint = ? ORDER BY s.updated_at DESC"#,
    )
    .bind(&fp)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| SessionListEntry {
            id: row.get(0),
            updated_at: row.get(1),
            message_count: row.get(2),
        })
        .collect())
}

pub async fn adaptive_stub_get(
    pool: &SqlitePool,
    key: &str,
) -> Result<Option<String>, SessionError> {
    let v: Option<String> = sqlx::query_scalar("SELECT value FROM adaptive_stub WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await?;
    Ok(v)
}

pub async fn adaptive_stub_set(
    pool: &SqlitePool,
    key: &str,
    value: &str,
) -> Result<(), SessionError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO adaptive_stub (key, value, updated_at) VALUES (?, ?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
    )
    .bind(key)
    .bind(value)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(())
}
