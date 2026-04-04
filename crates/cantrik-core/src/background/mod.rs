//! Background job queue in session SQLite (Sprint 12, PRD §4.3).

mod notify;

pub use notify::{NotificationChannels, notify_approval_needed};

use std::path::PathBuf;

use crate::config::BackgroundConfig;

use chrono::Utc;
use sqlx::Row;
use sqlx::SqlitePool;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobState {
    Queued,
    Running,
    WaitingApproval,
    Completed,
    Failed,
}

impl JobState {
    pub fn as_str(&self) -> &'static str {
        match self {
            JobState::Queued => "queued",
            JobState::Running => "running",
            JobState::WaitingApproval => "waiting_approval",
            JobState::Completed => "completed",
            JobState::Failed => "failed",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "queued" => Some(JobState::Queued),
            "running" => Some(JobState::Running),
            "waiting_approval" => Some(JobState::WaitingApproval),
            "completed" => Some(JobState::Completed),
            "failed" => Some(JobState::Failed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BackgroundJob {
    pub id: String,
    pub project_fingerprint: String,
    pub cwd: String,
    pub goal: String,
    pub state: JobState,
    pub last_error: Option<String>,
    pub approval_hint: Option<String>,
    pub rounds_done: i64,
    pub notify_on_approval: bool,
    pub created_at: String,
    pub updated_at: String,
    pub heartbeat_at: Option<String>,
}

#[derive(Debug, Error)]
pub enum BackgroundError {
    #[error("sql: {0}")]
    Sql(#[from] sqlx::Error),
    #[error("unknown job state: {0}")]
    BadState(String),
    #[error("{0}")]
    Other(String),
}

fn row_to_job(row: &sqlx::sqlite::SqliteRow) -> Result<BackgroundJob, BackgroundError> {
    let state_s: String = row.try_get("state")?;
    let state = JobState::parse(&state_s).ok_or_else(|| BackgroundError::BadState(state_s))?;
    let n: i64 = row.try_get("notify_on_approval")?;
    Ok(BackgroundJob {
        id: row.try_get("id")?,
        project_fingerprint: row.try_get("project_fingerprint")?,
        cwd: row.try_get("cwd")?,
        goal: row.try_get("goal")?,
        state,
        last_error: row.try_get("last_error")?,
        approval_hint: row.try_get("approval_hint")?,
        rounds_done: row.try_get("rounds_done")?,
        notify_on_approval: n != 0,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
        heartbeat_at: row.try_get("heartbeat_at")?,
    })
}

pub async fn enqueue_job(
    pool: &SqlitePool,
    project_fingerprint: &str,
    cwd: &str,
    goal: &str,
    notify_on_approval: bool,
) -> Result<String, BackgroundError> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let notify = if notify_on_approval { 1 } else { 0 };
    sqlx::query(
        r#"INSERT INTO background_jobs
        (id, project_fingerprint, cwd, goal, state, rounds_done, notify_on_approval, created_at, updated_at)
        VALUES (?, ?, ?, ?, 'queued', 0, ?, ?, ?)"#,
    )
    .bind(&id)
    .bind(project_fingerprint)
    .bind(cwd)
    .bind(goal)
    .bind(notify)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(id)
}

/// Atomically claim one queued job as `running` (race-safe for multiple daemons).
pub async fn claim_next_queued_job(
    pool: &SqlitePool,
) -> Result<Option<BackgroundJob>, BackgroundError> {
    let mut tx = pool.begin().await?;
    let row = sqlx::query(
        r#"SELECT id FROM background_jobs
           WHERE state = 'queued'
           ORDER BY created_at ASC
           LIMIT 1"#,
    )
    .fetch_optional(&mut *tx)
    .await?;

    let Some(row) = row else {
        tx.commit().await?;
        return Ok(None);
    };
    let id: String = row.try_get(0)?;
    let now = Utc::now().to_rfc3339();
    let n = sqlx::query(
        r#"UPDATE background_jobs
           SET state = 'running', updated_at = ?, heartbeat_at = ?
           WHERE id = ? AND state = 'queued'"#,
    )
    .bind(&now)
    .bind(&now)
    .bind(&id)
    .execute(&mut *tx)
    .await?
    .rows_affected();

    if n == 0 {
        tx.commit().await?;
        return Ok(None);
    }

    let row = sqlx::query(
        r#"SELECT id, project_fingerprint, cwd, goal, state, last_error, approval_hint,
                  rounds_done, notify_on_approval, created_at, updated_at, heartbeat_at
           FROM background_jobs WHERE id = ?"#,
    )
    .bind(&id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    row_to_job(&row).map(Some)
}

pub async fn update_job_state(
    pool: &SqlitePool,
    id: &str,
    state: JobState,
    last_error: Option<&str>,
    approval_hint: Option<&str>,
) -> Result<(), BackgroundError> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        r#"UPDATE background_jobs
           SET state = ?, updated_at = ?, last_error = ?, approval_hint = ?, heartbeat_at = ?
           WHERE id = ?"#,
    )
    .bind(state.as_str())
    .bind(&now)
    .bind(last_error)
    .bind(approval_hint)
    .bind(&now)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn set_rounds_done(
    pool: &SqlitePool,
    id: &str,
    rounds: i64,
) -> Result<(), BackgroundError> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "UPDATE background_jobs SET rounds_done = ?, updated_at = ?, heartbeat_at = ? WHERE id = ?",
    )
    .bind(rounds)
    .bind(&now)
    .bind(&now)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn touch_heartbeat(pool: &SqlitePool, id: &str) -> Result<(), BackgroundError> {
    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE background_jobs SET heartbeat_at = ?, updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// `waiting_approval` → `queued` so daemon can continue.
pub async fn resume_job(
    pool: &SqlitePool,
    id: &str,
    project_fingerprint: &str,
) -> Result<bool, BackgroundError> {
    let now = Utc::now().to_rfc3339();
    let n = sqlx::query(
        r#"UPDATE background_jobs
           SET state = 'queued', updated_at = ?, approval_hint = NULL
           WHERE id = ? AND project_fingerprint = ? AND state = 'waiting_approval'"#,
    )
    .bind(&now)
    .bind(id)
    .bind(project_fingerprint)
    .execute(pool)
    .await?
    .rows_affected();
    Ok(n > 0)
}

pub async fn list_jobs_for_project(
    pool: &SqlitePool,
    project_fingerprint: &str,
    limit: i64,
) -> Result<Vec<BackgroundJob>, BackgroundError> {
    let rows = sqlx::query(
        r#"SELECT id, project_fingerprint, cwd, goal, state, last_error, approval_hint,
                  rounds_done, notify_on_approval, created_at, updated_at, heartbeat_at
           FROM background_jobs
           WHERE project_fingerprint = ?
           ORDER BY updated_at DESC
           LIMIT ?"#,
    )
    .bind(project_fingerprint)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    rows.iter().map(row_to_job).collect()
}

pub async fn list_all_jobs(
    pool: &SqlitePool,
    limit: i64,
) -> Result<Vec<BackgroundJob>, BackgroundError> {
    let rows = sqlx::query(
        r#"SELECT id, project_fingerprint, cwd, goal, state, last_error, approval_hint,
                  rounds_done, notify_on_approval, created_at, updated_at, heartbeat_at
           FROM background_jobs
           ORDER BY updated_at DESC
           LIMIT ?"#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    rows.iter().map(row_to_job).collect()
}

/// Build notifier targets from merged config. If `job_wants_notify` is false, all channels are off.
pub fn notification_channels_from_config(
    c: &BackgroundConfig,
    job_wants_notify: bool,
) -> NotificationChannels {
    if !job_wants_notify {
        return NotificationChannels {
            desktop: false,
            webhook_url: None,
            flag_path: None,
        };
    }
    let flag = c
        .approval_flag_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| crate::session::share_dir().join("approval-pending.flag"));
    NotificationChannels {
        desktop: crate::config::effective_background_desktop_notify(c),
        webhook_url: c.webhook_url.clone(),
        flag_path: Some(flag),
    }
}

pub async fn get_job(
    pool: &SqlitePool,
    id: &str,
    project_fingerprint: Option<&str>,
) -> Result<Option<BackgroundJob>, BackgroundError> {
    let row = if let Some(fp) = project_fingerprint {
        sqlx::query(
            r#"SELECT id, project_fingerprint, cwd, goal, state, last_error, approval_hint,
                      rounds_done, notify_on_approval, created_at, updated_at, heartbeat_at
               FROM background_jobs WHERE id = ? AND project_fingerprint = ?"#,
        )
        .bind(id)
        .bind(fp)
        .fetch_optional(pool)
        .await?
    } else {
        sqlx::query(
            r#"SELECT id, project_fingerprint, cwd, goal, state, last_error, approval_hint,
                      rounds_done, notify_on_approval, created_at, updated_at, heartbeat_at
               FROM background_jobs WHERE id = ?"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
    };

    row.map(|r| row_to_job(&r)).transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::connect_pool;

    #[tokio::test]
    async fn enqueue_claim_complete() {
        let dir = std::env::temp_dir().join(format!("cantrik-bg-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        unsafe { std::env::set_var("CANTRIK_MEMORY_DB", dir.join("t.db")) };
        let pool = connect_pool().await.expect("pool");
        let id = enqueue_job(&pool, "fp1", "/tmp", "goal", true)
            .await
            .expect("enqueue");
        let j = claim_next_queued_job(&pool)
            .await
            .expect("claim")
            .expect("some");
        assert_eq!(j.id, id);
        assert_eq!(j.state, JobState::Running);
        update_job_state(&pool, &id, JobState::Completed, None, None)
            .await
            .unwrap();
        let list = list_jobs_for_project(&pool, "fp1", 10).await.unwrap();
        assert_eq!(list[0].state, JobState::Completed);
        unsafe { std::env::remove_var("CANTRIK_MEMORY_DB") };
        let _ = std::fs::remove_dir_all(&dir);
    }
}
