//! Approximate LLM cost rows in the session SQLite DB (Sprint 14).

use sqlx::SqlitePool;
use thiserror::Error;

/// Current UTC month key `YYYY-MM` for [`month_spend_usd`].
pub fn current_year_month_utc() -> String {
    chrono::Utc::now().format("%Y-%m").to_string()
}

#[derive(Debug, Error)]
pub enum UsageLedgerError {
    #[error(transparent)]
    Sql(#[from] sqlx::Error),
}

/// Sum `cost_usd_approx` for this session within the project fingerprint.
pub async fn session_spend_usd(
    pool: &SqlitePool,
    project_fingerprint: &str,
    session_id: &str,
) -> Result<f64, UsageLedgerError> {
    let v: Option<f64> = sqlx::query_scalar(
        "SELECT COALESCE(SUM(cost_usd_approx), 0.0) FROM llm_usage WHERE project_fingerprint = ? AND session_id = ?",
    )
    .bind(project_fingerprint)
    .bind(session_id)
    .fetch_one(pool)
    .await?;
    Ok(v.unwrap_or(0.0))
}

/// Sum for UTC calendar month `YYYY-MM` (compared via `strftime` on `at`).
pub async fn month_spend_usd(
    pool: &SqlitePool,
    project_fingerprint: &str,
    year_month: &str,
) -> Result<f64, UsageLedgerError> {
    let v: Option<f64> = sqlx::query_scalar(
        "SELECT COALESCE(SUM(cost_usd_approx), 0.0) FROM llm_usage WHERE project_fingerprint = ? AND strftime('%Y-%m', at) = ?",
    )
    .bind(project_fingerprint)
    .bind(year_month)
    .fetch_one(pool)
    .await?;
    Ok(v.unwrap_or(0.0))
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_llm_usage(
    pool: &SqlitePool,
    session_id: Option<&str>,
    project_fingerprint: &str,
    provider: &str,
    model: &str,
    tier: Option<&str>,
    input_chars: i64,
    output_chars: i64,
    cost_usd_approx: f64,
) -> Result<(), UsageLedgerError> {
    let at = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        r#"INSERT INTO llm_usage (session_id, project_fingerprint, at, provider, model, tier, input_chars, output_chars, cost_usd_approx)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(session_id)
    .bind(project_fingerprint)
    .bind(&at)
    .bind(provider)
    .bind(model)
    .bind(tier)
    .bind(input_chars)
    .bind(output_chars)
    .bind(cost_usd_approx)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::connect_pool;
    use std::fs;

    #[tokio::test]
    async fn aggregate_session_and_month() {
        let dir = std::env::temp_dir().join(format!("cantrik-usage-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        unsafe {
            std::env::set_var(crate::session::ENV_MEMORY_DB, dir.join("mem.sqlite"));
        }
        let pool = connect_pool().await.expect("pool");
        let fp = "fp_test";
        sqlx::query(
            "INSERT INTO sessions (id, project_fingerprint, created_at, updated_at) VALUES ('s1', ?, datetime('now'), datetime('now'))",
        )
        .bind(fp)
        .execute(&pool)
        .await
        .expect("session");

        insert_llm_usage(
            &pool,
            Some("s1"),
            fp,
            "openai",
            "gpt-4o-mini",
            Some("simple"),
            100,
            50,
            0.01,
        )
        .await
        .expect("ins");
        insert_llm_usage(
            &pool,
            Some("s1"),
            fp,
            "openai",
            "gpt-4o-mini",
            None,
            200,
            100,
            0.02,
        )
        .await
        .expect("ins2");

        let s = session_spend_usd(&pool, fp, "s1").await.expect("sess");
        assert!((s - 0.03).abs() < 1e-9);

        let ym = chrono::Utc::now().format("%Y-%m").to_string();
        let m = month_spend_usd(&pool, fp, &ym).await.expect("month");
        assert!((m - 0.03).abs() < 1e-9);

        unsafe {
            std::env::remove_var(crate::session::ENV_MEMORY_DB);
        }
        let _ = fs::remove_dir_all(&dir);
    }
}
