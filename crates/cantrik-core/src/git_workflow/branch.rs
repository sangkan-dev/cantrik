use std::path::Path;

use super::run::{GitWorkflowError, git_write};

pub fn sanitize_task_slug(raw: &str) -> String {
    let mut s = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '-'
            }
        })
        .collect::<String>();
    while s.contains("--") {
        s = s.replace("--", "-");
    }
    s.trim_matches('-').to_string()
}

pub fn is_worktree_dirty(cwd: &Path) -> Result<bool, GitWorkflowError> {
    let out = git_write(cwd, &["status", "--porcelain"])?;
    let text = String::from_utf8_lossy(&out.stdout);
    Ok(!text.trim().is_empty())
}

pub fn create_feature_branch(
    cwd: &Path,
    prefix: &str,
    slug: &str,
    allow_dirty: bool,
) -> Result<String, GitWorkflowError> {
    let slug = sanitize_task_slug(slug);
    if slug.is_empty() {
        return Err(GitWorkflowError::Failed {
            code: None,
            stderr: "task slug is empty after sanitization".into(),
        });
    }
    if !allow_dirty && is_worktree_dirty(cwd)? {
        return Err(GitWorkflowError::Failed {
            code: None,
            stderr: "working tree has changes; commit/stash or pass --allow-dirty".into(),
        });
    }
    let branch = format!("{prefix}{slug}");
    git_write(cwd, &["checkout", "-b", &branch])?;
    Ok(branch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_sanitizes() {
        assert_eq!(sanitize_task_slug("Refactor Auth!!!"), "refactor-auth");
        assert_eq!(sanitize_task_slug("  x  "), "x");
    }
}
