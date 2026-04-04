use std::path::Path;
use std::process::Command;

#[derive(Debug)]
pub enum GitWorkflowError {
    Io(std::io::Error),
    Failed { code: Option<i32>, stderr: String },
}

impl std::fmt::Display for GitWorkflowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitWorkflowError::Io(e) => write!(f, "{e}"),
            GitWorkflowError::Failed { code, stderr } => {
                write!(f, "git failed (code {:?}): {stderr}", code)
            }
        }
    }
}

impl std::error::Error for GitWorkflowError {}

impl From<std::io::Error> for GitWorkflowError {
    fn from(e: std::io::Error) -> Self {
        GitWorkflowError::Io(e)
    }
}

/// Run `git` in `cwd` for write-capable workflows (not the read-only agent allowlist).
pub fn git_write(cwd: &Path, args: &[&str]) -> Result<std::process::Output, GitWorkflowError> {
    let out = Command::new("git").current_dir(cwd).args(args).output()?;
    if !out.status.success() {
        return Err(GitWorkflowError::Failed {
            code: out.status.code(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        });
    }
    Ok(out)
}
