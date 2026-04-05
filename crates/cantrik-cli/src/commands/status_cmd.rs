//! `cantrik status` — list background jobs for this project (Sprint 12).

use std::fs;
use std::path::Path;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use cantrik_core::background::{BackgroundJob, JobState, list_all_jobs, list_jobs_for_project};
use cantrik_core::session::{connect_pool, project_fingerprint};
use serde_json::json;

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

fn job_json(j: &BackgroundJob) -> serde_json::Value {
    json!({
        "id": j.id,
        "project_fingerprint": j.project_fingerprint,
        "cwd": j.cwd,
        "goal": j.goal,
        "state": j.state.as_str(),
        "last_error": j.last_error,
        "approval_hint": j.approval_hint,
        "rounds_done": j.rounds_done,
        "notify_on_approval": j.notify_on_approval,
        "created_at": j.created_at,
        "updated_at": j.updated_at,
        "heartbeat_at": j.heartbeat_at,
    })
}

fn write_harness_summary_file(cwd: &Path, rows: &[serde_json::Value]) -> Result<(), String> {
    let dir = cwd.join(".cantrik");
    fs::create_dir_all(&dir).map_err(|e| format!("create .cantrik: {e}"))?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let payload = json!({
        "generated_at_unix": ts,
        "cwd": cwd.display().to_string(),
        "jobs": rows,
    });
    let path = dir.join("session-harness-summary.json");
    let body = serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?;
    fs::write(&path, body).map_err(|e| format!("write {}: {e}", path.display()))?;
    Ok(())
}

pub async fn run(
    cwd: &Path,
    all_projects: bool,
    limit: i64,
    json_out: bool,
    write_harness_summary: bool,
) -> ExitCode {
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

    let rows: Vec<serde_json::Value> = jobs.iter().map(job_json).collect();

    if write_harness_summary && let Err(e) = write_harness_summary_file(cwd, &rows) {
        eprintln!("cantrik status: harness summary: {e}");
        return ExitCode::FAILURE;
    }

    if jobs.is_empty() {
        if json_out {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({ "jobs": [] }))
                    .unwrap_or_else(|_| { "{\"jobs\":[]}".to_string() })
            );
        } else {
            println!("(no background jobs)");
        }
        return ExitCode::SUCCESS;
    }

    if json_out {
        match serde_json::to_string_pretty(&json!({ "jobs": rows })) {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("cantrik status: json: {e}");
                return ExitCode::FAILURE;
            }
        }
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
