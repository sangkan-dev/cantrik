use std::path::Path;
use std::process::ExitCode;

use cantrik_core::config::load_merged_config;
use cantrik_core::git_workflow::{build_review_prompt, diff_staged, diff_worktree};
use cantrik_core::llm;

pub async fn run(cwd: &Path, worktree: bool, soft: bool) -> ExitCode {
    let config = match load_merged_config(cwd) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("failed to load config: {e}");
            return ExitCode::FAILURE;
        }
    };
    let use_worktree = worktree;
    let diff = if use_worktree {
        match diff_worktree(cwd) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("review: {e}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        match diff_staged(cwd) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("review: {e}");
                return ExitCode::FAILURE;
            }
        }
    };
    if diff.trim().is_empty() {
        eprintln!("review: empty diff");
        return ExitCode::from(2);
    }
    let prompt = build_review_prompt(&diff, !use_worktree);
    match llm::ask_complete_text(&config, &prompt).await {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            if soft {
                eprintln!("review: LLM error (--soft): {e}");
                ExitCode::SUCCESS
            } else {
                eprintln!("review: {e}");
                ExitCode::FAILURE
            }
        }
    }
}
