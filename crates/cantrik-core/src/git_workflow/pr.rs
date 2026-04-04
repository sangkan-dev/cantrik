use std::path::Path;
use std::process::Command;

use super::run::{GitWorkflowError, git_write};

/// Best-effort: `git remote get-url origin`.
pub fn origin_url(cwd: &Path) -> Result<String, GitWorkflowError> {
    let out = git_write(cwd, &["remote", "get-url", "origin"])?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

pub fn looks_like_github_origin(url: &str) -> bool {
    let u = url.to_ascii_lowercase();
    u.contains("github.com") || u.contains("github:")
}

pub fn gh_available() -> bool {
    Command::new("gh")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn pr_create_dry_run_hint() -> &'static str {
    "Dry run: review title/body, then re-run with --approve to execute `gh pr create`."
}

/// Spawn `gh pr create` in `cwd`. Requires GitHub CLI authenticated.
pub fn run_gh_pr_create(
    cwd: &Path,
    title: Option<&str>,
    body: Option<&str>,
    body_file: Option<&Path>,
    draft: bool,
) -> Result<std::process::Output, GitWorkflowError> {
    let mut cmd = Command::new("gh");
    cmd.current_dir(cwd);
    cmd.args(["pr", "create"]);
    if draft {
        cmd.arg("--draft");
    }
    if let Some(t) = title.filter(|s| !s.is_empty()) {
        cmd.args(["--title", t]);
    }
    if let Some(p) = body_file {
        let ps = p.to_str().unwrap_or("");
        cmd.args(["--body-file", ps]);
    } else if let Some(b) = body.filter(|s| !s.is_empty()) {
        cmd.args(["--body", b]);
    } else {
        cmd.arg("--fill");
    }
    let out = cmd.output()?;
    if !out.status.success() {
        return Err(GitWorkflowError::Failed {
            code: out.status.code(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        });
    }
    Ok(out)
}
