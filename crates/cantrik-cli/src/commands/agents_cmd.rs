use std::path::Path;
use std::process::ExitCode;

use cantrik_core::config::AppConfig;
use cantrik_core::multi_agent::{OrchestratorOptions, run_orchestrated};

pub(crate) async fn run(
    config: &AppConfig,
    _cwd: &Path,
    goal: &str,
    dry_run: bool,
    max_parallel: Option<usize>,
) -> ExitCode {
    if goal.trim().is_empty() {
        eprintln!("agents: empty goal");
        return ExitCode::FAILURE;
    }

    let opts = OrchestratorOptions {
        depth: 0,
        dry_run,
        max_parallel_override: max_parallel,
    };

    match run_orchestrated(config, goal.trim(), opts).await {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("agents: {e}");
            ExitCode::FAILURE
        }
    }
}
