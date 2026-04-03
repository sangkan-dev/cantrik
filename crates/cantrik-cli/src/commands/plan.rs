use std::process::ExitCode;

use cantrik_core::config::AppConfig;

pub(crate) fn run(_config: &AppConfig, task: &str) -> ExitCode {
    if task.trim().is_empty() {
        eprintln!("plan: empty task");
        return ExitCode::from(1);
    }
    println!(
        "plan: (scaffold) planner not wired — task:\n{}",
        task.trim()
    );
    ExitCode::SUCCESS
}
