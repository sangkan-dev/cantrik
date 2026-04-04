//! `cantrik status` — list background jobs for this project (Sprint 12).

use std::path::Path;
use std::process::ExitCode;

use cantrik_core::background::{JobState, list_all_jobs, list_jobs_for_project};
use cantrik_core::session::{connect_pool, project_fingerprint};

fn state_label(s: JobState) -> &'static str {
    match s {
        JobState::Queued => "queued",
        JobState::Running => "running",
        JobState::WaitingApproval => "waiting_approval",
        JobState::Completed => "completed",
        JobState::Failed => "failed",
    }
}

fn trunc(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let t: String = s.chars().take(max.saturating_sub(3)).collect();
    format!("{t}...")
}

pub async fn run(cwd: &Path, all_projects: bool, limit: i64) -> ExitCode {
    let pool = match connect_pool().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("cantrik status: database: {e}");
            return ExitCode::FAILURE;
        }
    };

    let jobs = if all_projects {
        match list_all_jobs(&pool, limit).await {
            Ok(j) => j,
            Err(e) => {
                eprintln!("cantrik status: {e}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        let fp = project_fingerprint(cwd);
        match list_jobs_for_project(&pool, &fp, limit).await {
            Ok(j) => j,
            Err(e) => {
                eprintln!("cantrik status: {e}");
                return ExitCode::FAILURE;
            }
        }
    };

    if jobs.is_empty() {
        println!("(no background jobs)");
        return ExitCode::SUCCESS;
    }

    for j in jobs {
        let goal = trunc(&j.goal, 72);
        println!(
            "{}  {}  {}  {}",
            j.id,
            state_label(j.state),
            j.updated_at,
            goal
        );
        if let Some(h) = j.approval_hint.as_ref() {
            println!("    hint: {h}");
        }
        if let Some(err) = j.last_error.as_ref() {
            let e = trunc(err, 120);
            println!("    error: {e}");
        }
    }

    ExitCode::SUCCESS
}
