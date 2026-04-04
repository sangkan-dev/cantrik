//! `cantrik background …` — enqueue or resume jobs (Sprint 12).

use std::path::Path;
use std::process::ExitCode;

use cantrik_core::background::{enqueue_job, resume_job};
use cantrik_core::session::{connect_pool, project_fingerprint};

pub async fn run(cwd: &Path, notify: bool, args: &[String]) -> ExitCode {
    if args.is_empty() {
        eprintln!("cantrik background: missing goal or `resume <job-id>`");
        return ExitCode::from(2);
    }

    if args[0] == "resume" {
        if args.len() < 2 {
            eprintln!("cantrik background resume: missing job id");
            return ExitCode::from(2);
        }
        let id = &args[1];
        return resume_by_id(cwd, id).await;
    }

    let goal = args.join(" ");
    let pool = match connect_pool().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("cantrik background: database: {e}");
            return ExitCode::FAILURE;
        }
    };
    let fp = project_fingerprint(cwd);
    let cwd_s = cwd.to_string_lossy().to_string();
    match enqueue_job(&pool, &fp, &cwd_s, &goal, notify).await {
        Ok(id) => {
            println!("queued job {id}");
            println!(
                "Run `cantrik daemon` (or install a user systemd/launchd unit; see contrib/)."
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("cantrik background: {e}");
            ExitCode::FAILURE
        }
    }
}

async fn resume_by_id(cwd: &Path, id: &str) -> ExitCode {
    let pool = match connect_pool().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("cantrik background resume: database: {e}");
            return ExitCode::FAILURE;
        }
    };
    let fp = project_fingerprint(cwd);
    match resume_job(&pool, id, &fp).await {
        Ok(true) => {
            println!("job {id} re-queued (waiting_approval → queued)");
            ExitCode::SUCCESS
        }
        Ok(false) => {
            eprintln!(
                "cantrik background resume: no job {id} in waiting_approval for this project"
            );
            ExitCode::from(1)
        }
        Err(e) => {
            eprintln!("cantrik background resume: {e}");
            ExitCode::FAILURE
        }
    }
}
