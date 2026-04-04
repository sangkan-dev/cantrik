//! `cantrik cost` — approximate LLM spend from `llm_usage` (Sprint 14).

use std::path::Path;
use std::process::ExitCode;

use cantrik_core::config::load_merged_config;
use cantrik_core::session::{connect_pool, open_or_create_session, session_project_fingerprint};
use cantrik_core::usage_ledger::{current_year_month_utc, month_spend_usd, session_spend_usd};

pub async fn run(cwd: &Path, session_only: bool) -> ExitCode {
    let pool = match connect_pool().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("cost: database: {e}");
            return ExitCode::FAILURE;
        }
    };
    let app = load_merged_config(cwd).unwrap_or_default();
    let fp = session_project_fingerprint(cwd, &app);
    let ym = current_year_month_utc();

    let month = match month_spend_usd(&pool, &fp, &ym).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("cost: {e}");
            return ExitCode::FAILURE;
        }
    };

    if session_only {
        let sid = match open_or_create_session(&pool, cwd).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("cost: session: {e}");
                return ExitCode::FAILURE;
            }
        };
        let s = match session_spend_usd(&pool, &fp, &sid).await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("cost: {e}");
                return ExitCode::FAILURE;
            }
        };
        println!("approx session spend: {s:.6} USD");
        return ExitCode::SUCCESS;
    }

    println!("approx month ({ym}, UTC, this project): {month:.6} USD");
    match open_or_create_session(&pool, cwd).await {
        Ok(sid) => match session_spend_usd(&pool, &fp, &sid).await {
            Ok(s) => println!("approx active session spend: {s:.6} USD"),
            Err(e) => eprintln!("cost: session aggregate: {e}"),
        },
        Err(e) => eprintln!("cost: session: {e}"),
    }
    ExitCode::SUCCESS
}
