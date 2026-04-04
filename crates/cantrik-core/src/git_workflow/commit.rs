use std::path::Path;

use crate::config::AppConfig;
use crate::llm::{self, LlmError};

use super::run::{GitWorkflowError, git_write};

pub const COMMIT_MSG_SYSTEM: &str = r#"You write Git commit messages for the following unified diff.
Rules:
- Use Conventional Commits when appropriate (type(scope): subject).
- First line <= 72 characters; optional body after a blank line.
- Describe what changed and why, not implementation trivia.
- Output ONLY the commit message text, no markdown fences or commentary."#;

pub fn diff_staged(cwd: &Path) -> Result<String, GitWorkflowError> {
    let out = git_write(cwd, &["diff", "--cached"])?;
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

pub fn diff_worktree(cwd: &Path) -> Result<String, GitWorkflowError> {
    let out = git_write(cwd, &["diff", "HEAD"])?;
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn build_commit_prompt(diff: &str) -> String {
    format!(
        "{system}\n\n## Diff\n\n```diff\n{diff}\n```\n",
        system = COMMIT_MSG_SYSTEM,
        diff = diff
    )
}

pub async fn propose_commit_message(app: &AppConfig, diff: &str) -> Result<String, LlmError> {
    let prompt = build_commit_prompt(diff);
    llm::ask_complete_text(app, &prompt).await
}

pub fn git_commit_with_message(
    cwd: &Path,
    message: &str,
    amend: bool,
    no_verify: bool,
) -> Result<(), GitWorkflowError> {
    let mut argv: Vec<String> = vec!["commit".into(), "-m".into(), message.into()];
    if amend {
        argv.push("--amend".into());
    }
    if no_verify {
        argv.push("--no-verify".into());
    }
    let refs: Vec<&str> = argv.iter().map(|s| s.as_str()).collect();
    git_write(cwd, &refs)?;
    Ok(())
}
