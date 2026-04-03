use std::process::ExitCode;

use cantrik_core::config::AppConfig;

/// Scaffold: print stub; Sprint 3+ hooks LLM / retrieval here.
pub(crate) fn run(_config: &AppConfig, prompt: &str) -> ExitCode {
    if prompt.trim().is_empty() {
        eprintln!("ask: empty prompt");
        return ExitCode::from(1);
    }
    println!(
        "ask: (scaffold) not yet connected to LLM/index — prompt:\n{}",
        prompt.trim()
    );
    ExitCode::SUCCESS
}
