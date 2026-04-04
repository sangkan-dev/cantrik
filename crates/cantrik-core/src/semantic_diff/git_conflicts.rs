use std::path::Path;

use super::git_workspace::{GitWorkspaceError, status_porcelain};

#[derive(Debug, Clone)]
pub struct ConflictEntry {
    pub xy: String,
    pub path: String,
}

pub fn list_conflicts(cwd: &Path) -> Result<(Vec<ConflictEntry>, Vec<String>), GitWorkspaceError> {
    let text = status_porcelain(cwd)?;
    let mut conflicts = Vec::new();
    let mut hints = Vec::new();

    for line in text.lines() {
        let line = line.trim_end();
        if line.len() < 4 {
            continue;
        }
        let xy = &line[0..2];
        let rest = line[2..].trim_start();
        if xy.contains('U') || xy == "AA" || xy == "DD" {
            conflicts.push(ConflictEntry {
                xy: xy.to_string(),
                path: rest.to_string(),
            });
        }
    }

    if !conflicts.is_empty() {
        hints.push(
            "Open each path and search for conflict markers: <<<<<<<, =======, >>>>>>>".into(),
        );
        hints.push("After editing, `git add <path>` then commit or continue merge/rebase.".into());
    }

    Ok((conflicts, hints))
}
