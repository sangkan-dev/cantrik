//! `git blame` + `git log` context for `cantrik explain`.

use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

use super::IntelligenceError;

fn git_toplevel(repo: &Path) -> Result<std::path::PathBuf, IntelligenceError> {
    let out = Command::new("git")
        .current_dir(repo)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|e| IntelligenceError::Git(format!("failed to spawn git: {e}")))?;
    if !out.status.success() {
        return Err(IntelligenceError::Git(
            String::from_utf8_lossy(&out.stderr).trim().to_string(),
        ));
    }
    let s = String::from_utf8_lossy(&out.stdout);
    Ok(Path::new(s.trim()).to_path_buf())
}

fn rel_path_in_repo(repo: &Path, file: &Path) -> Result<String, IntelligenceError> {
    let top = git_toplevel(repo)?;
    let abs_file = if file.is_absolute() {
        file.to_path_buf()
    } else {
        repo.join(file)
    };
    let abs_file = abs_file
        .canonicalize()
        .map_err(|e| IntelligenceError::Msg(format!("cannot resolve file path: {e}")))?;
    let rel = abs_file
        .strip_prefix(&top)
        .map_err(|_| {
            IntelligenceError::Msg(format!(
                "file {} is outside git repository root {}",
                abs_file.display(),
                top.display()
            ))
        })?
        .to_string_lossy()
        .replace('\\', "/");
    Ok(rel)
}

/// First-column commit prefixes from default `git blame` lines (7+ hex).
pub fn extract_blame_commit_prefixes(blame_stdout: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for line in blame_stdout.lines() {
        let token = line.split_whitespace().next().unwrap_or("");
        if token.len() >= 7 && token.chars().all(|c| c.is_ascii_hexdigit()) {
            let short = token.chars().take(12).collect::<String>();
            if seen.insert(short.clone()) {
                out.push(short);
            }
        }
    }
    out
}

fn run_git_blame(
    repo: &Path,
    rel: &str,
    line: Option<u32>,
    max_lines: usize,
) -> Result<String, IntelligenceError> {
    let mut cmd = Command::new("git");
    cmd.current_dir(repo).arg("blame").arg("-w");
    if let Some(l) = line {
        if l == 0 {
            return Err(IntelligenceError::Msg("line must be >= 1".into()));
        }
        // Window around the line of interest.
        let span = max_lines.clamp(1, 500) as u32;
        cmd.arg("-L").arg(format!("{},+{}", l, span));
    }
    cmd.arg("--").arg(rel);
    let out = cmd
        .output()
        .map_err(|e| IntelligenceError::Git(format!("git blame: {e}")))?;
    if !out.status.success() {
        return Err(IntelligenceError::Git(
            String::from_utf8_lossy(&out.stderr).trim().to_string(),
        ));
    }
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    if line.is_none() {
        let mut lines: Vec<_> = text.lines().take(max_lines).collect();
        if lines.len() < text.lines().count() {
            lines.push("… (truncated)");
        }
        return Ok(lines.join("\n"));
    }
    Ok(text)
}

fn run_git_log_file(repo: &Path, rel: &str, n: u32) -> Result<String, IntelligenceError> {
    let out = Command::new("git")
        .current_dir(repo)
        .args([
            "log",
            "-n",
            &n.to_string(),
            "--date=short",
            "--pretty=format:%h %ad %s",
            "--",
            rel,
        ])
        .output()
        .map_err(|e| IntelligenceError::Git(format!("git log: {e}")))?;
    if !out.status.success() {
        return Err(IntelligenceError::Git(
            String::from_utf8_lossy(&out.stderr).trim().to_string(),
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn run_git_show_commits(
    repo: &Path,
    prefixes: &[String],
    max: usize,
) -> Result<String, IntelligenceError> {
    let mut buf = String::new();
    for p in prefixes.iter().take(max) {
        let out = Command::new("git")
            .current_dir(repo)
            .args([
                "show",
                "-s",
                "--date=short",
                "--pretty=format:%H%n%ad%n%s%n%b",
                p,
            ])
            .output()
            .map_err(|e| IntelligenceError::Git(format!("git show: {e}")))?;
        if out.status.success() {
            buf.push_str(&String::from_utf8_lossy(&out.stdout));
            buf.push_str("\n---\n");
        }
    }
    Ok(buf)
}

/// Raw blame + log (+ short summaries for blame commits) for display or LLM.
pub fn collect_explain_context(
    repo: &Path,
    file: &Path,
    line: Option<u32>,
    max_blame_lines: usize,
    log_limit: u32,
) -> Result<String, IntelligenceError> {
    let rel = rel_path_in_repo(repo, file)?;
    let blame = run_git_blame(repo, &rel, line, max_blame_lines)?;
    let log = run_git_log_file(repo, &rel, log_limit)?;
    let prefixes = extract_blame_commit_prefixes(&blame);
    let shows = run_git_show_commits(repo, &prefixes, 8)?;

    let mut s = String::new();
    s.push_str("## git blame\n");
    s.push_str(&blame);
    s.push_str("\n\n## git log (file)\n");
    s.push_str(&log);
    if !shows.trim().is_empty() {
        s.push_str("\n\n## commits referenced in blame (summary)\n");
        s.push_str(&shows);
    }
    Ok(s)
}

pub fn build_explain_why_prompt(context: &str, rel_display: &str) -> String {
    format!(
        "You are a senior engineer. Using ONLY the git blame and log context below, explain how the code at `{rel_display}` evolved and why the current lines likely look the way they do. Be concise; cite commit subjects when useful. If context is insufficient, say what is missing.\n\n---\n{context}\n---",
        rel_display = rel_display,
        context = context
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_prefixes_from_blame() {
        let sample =
            "a1b2c3d (Alice 2024-01-01 1) fn main() {}\ne4f5a6b (Bob 2024-01-02 2) println!();";
        let p = extract_blame_commit_prefixes(sample);
        assert!(p.contains(&"a1b2c3d".to_string()));
        assert!(p.contains(&"e4f5a6b".to_string()));
    }
}
