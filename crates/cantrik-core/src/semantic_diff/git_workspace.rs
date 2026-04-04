use std::path::Path;
use std::process::Command;

#[derive(Debug)]
pub enum GitWorkspaceError {
    Io(std::io::Error),
    Failed { code: i32, stderr: String },
}

impl std::fmt::Display for GitWorkspaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitWorkspaceError::Io(e) => write!(f, "{e}"),
            GitWorkspaceError::Failed { code, stderr } => {
                write!(f, "git exit {code}: {stderr}")
            }
        }
    }
}

impl std::error::Error for GitWorkspaceError {}

impl From<std::io::Error> for GitWorkspaceError {
    fn from(e: std::io::Error) -> Self {
        GitWorkspaceError::Io(e)
    }
}

fn git_output(cwd: &Path, args: &[&str]) -> Result<Vec<u8>, GitWorkspaceError> {
    let out = Command::new("git").current_dir(cwd).args(args).output()?;
    if !out.status.success() {
        return Err(GitWorkspaceError::Failed {
            code: out.status.code().unwrap_or(-1),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        });
    }
    Ok(out.stdout)
}

pub fn changed_paths(cwd: &Path, staged: bool) -> Result<Vec<String>, GitWorkspaceError> {
    let args: &[&str] = if staged {
        &["diff", "--cached", "--name-only", "-z"]
    } else {
        &["diff", "--name-only", "-z"]
    };
    let raw = git_output(cwd, args)?;
    let mut paths = Vec::new();
    for p in raw.split(|b| *b == 0) {
        if p.is_empty() {
            continue;
        }
        let s = String::from_utf8_lossy(p).replace('\\', "/");
        if !s.is_empty() {
            paths.push(s);
        }
    }
    Ok(paths)
}

pub fn show_blob(cwd: &Path, rev_path: &str) -> Result<Option<String>, GitWorkspaceError> {
    let out = Command::new("git")
        .current_dir(cwd)
        .args(["show", rev_path])
        .output()?;
    if out.status.success() {
        return Ok(Some(String::from_utf8_lossy(&out.stdout).into_owned()));
    }
    let stderr = String::from_utf8_lossy(&out.stderr);
    if stderr.contains("does not exist")
        || stderr.contains("exists on disk, but not in")
        || stderr.contains("fatal: path")
    {
        return Ok(None);
    }
    Err(GitWorkspaceError::Failed {
        code: out.status.code().unwrap_or(-1),
        stderr: stderr.into_owned(),
    })
}

pub fn status_porcelain(cwd: &Path) -> Result<String, GitWorkspaceError> {
    let raw = git_output(cwd, &["status", "--porcelain"])?;
    Ok(String::from_utf8_lossy(&raw).into_owned())
}
