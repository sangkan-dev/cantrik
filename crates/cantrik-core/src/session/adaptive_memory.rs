//! Adaptive Begawan — store recent approval decisions and inject a short hint into prompts (Sprint 19, PRD §4.15).

use sqlx::SqlitePool;

use super::SessionError;

/// Insert one decision (best-effort; callers may ignore errors for non-critical telemetry).
pub async fn record_approval_memory(
    pool: &SqlitePool,
    project_fingerprint: &str,
    tool_id: &str,
    approved: bool,
    summary: &str,
) -> Result<(), SessionError> {
    let now = chrono::Utc::now().to_rfc3339();
    let a: i32 = if approved { 1 } else { 0 };
    let sum: String = summary.chars().take(512).collect();
    sqlx::query(
        r#"INSERT INTO approval_memory (project_fingerprint, tool_id, approved, summary, created_at)
           VALUES (?, ?, ?, ?, ?)"#,
    )
    .bind(project_fingerprint)
    .bind(tool_id)
    .bind(a)
    .bind(sum)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

/// Text block for prompt injection, or empty if disabled / no rows.
pub async fn adaptive_memory_prompt_addon(
    pool: &SqlitePool,
    project_fingerprint: &str,
    max_chars: usize,
) -> Result<String, SessionError> {
    let rows: Vec<(String, i32, String)> = sqlx::query_as(
        r#"SELECT tool_id, approved, summary FROM approval_memory
           WHERE project_fingerprint = ?
           ORDER BY id DESC
           LIMIT 16"#,
    )
    .bind(project_fingerprint)
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        return Ok(String::new());
    }

    let mut lines = vec![
        "Recent explicit user decisions on tool use (align suggestions accordingly; do not treat as permission to bypass guardrails):".to_string(),
    ];
    for (tid, ap, sum) in rows {
        let tag = if ap != 0 { "approved" } else { "rejected" };
        lines.push(format!("- {tag} `{tid}`: {sum}"));
    }
    let mut s = lines.join("\n");
    if s.len() > max_chars {
        s = s.chars().take(max_chars.saturating_sub(20)).collect();
        s.push_str("\n...[truncated]");
    }
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::connect_pool;

    #[tokio::test]
    async fn record_and_fetch_addon() {
        let dir = std::env::temp_dir().join(format!("cantrik-adapt-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        unsafe { std::env::set_var(crate::session::ENV_MEMORY_DB, dir.join("db.sqlite")) };

        let pool = connect_pool().await.expect("pool");
        record_approval_memory(&pool, "fp_test", "write_file", true, "wrote src/lib.rs")
            .await
            .expect("insert");
        let block = adaptive_memory_prompt_addon(&pool, "fp_test", 2000)
            .await
            .expect("addon");
        assert!(block.contains("write_file"));
        assert!(block.contains("approved"));

        unsafe {
            std::env::remove_var(crate::session::ENV_MEMORY_DB);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
