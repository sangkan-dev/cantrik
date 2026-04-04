//! Load `.cantrik/rules.md` and skill files from disk (Sprint 13).

use std::fs;
use std::path::{Path, PathBuf};

/// Skip injecting project rules (tests).
pub const ENV_NO_RULES: &str = "CANTRIK_NO_RULES";

pub fn project_rules_path(cwd: &Path) -> PathBuf {
    cwd.join(".cantrik").join("rules.md")
}

pub fn skills_dir(cwd: &Path) -> PathBuf {
    cwd.join(".cantrik").join("skills")
}

/// Raw text of `rules.md` if present and env allows.
pub fn load_rules_text(cwd: &Path) -> Option<String> {
    if std::env::var(ENV_NO_RULES).is_ok() {
        return None;
    }
    let p = project_rules_path(cwd);
    if !p.is_file() {
        return None;
    }
    fs::read_to_string(&p).ok()
}

/// All `*.md` files directly under `.cantrik/skills/` (non-recursive MVP).
pub fn list_skill_paths(cwd: &Path) -> Vec<PathBuf> {
    let dir = skills_dir(cwd);
    let Ok(entries) = fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "md"))
        .collect();
    paths.sort();
    paths
}

pub fn read_skill_file(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

/// First ~800 chars of file for scoring (headings + name).
pub fn skill_excerpt_for_scoring(content: &str) -> String {
    content.chars().take(800).collect()
}
