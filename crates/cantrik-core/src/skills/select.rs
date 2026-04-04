//! Keyword overlap scoring for skill selection (Sprint 13 MVP).

use std::collections::HashSet;
use std::path::Path;

use crate::config::{AppConfig, effective_skills_max_files, effective_skills_max_total_chars};

use super::load::{list_skill_paths, read_skill_file, skill_excerpt_for_scoring};

fn tokenize_lower(s: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    let mut cur = String::new();
    for c in s.chars() {
        if c.is_alphanumeric() || c == '_' || c == '-' {
            cur.push(c.to_ascii_lowercase());
        } else if !cur.is_empty() {
            if cur.len() > 1 {
                out.insert(std::mem::take(&mut cur));
            } else {
                cur.clear();
            }
        }
    }
    if cur.len() > 1 {
        out.insert(cur);
    }
    out
}

fn stem_file_name(path: &Path) -> String {
    path.file_stem()
        .map(|s| s.to_string_lossy().replace(['-', '_'], " "))
        .unwrap_or_default()
}

fn score_skill(user_tokens: &HashSet<String>, path: &Path, body_excerpt: &str) -> u32 {
    let mut score: u32 = 0;
    let name = stem_file_name(path);
    for t in tokenize_lower(&name) {
        if user_tokens.contains(&t) {
            score = score.saturating_add(3);
        }
    }
    for t in tokenize_lower(body_excerpt) {
        if user_tokens.contains(&t) {
            score = score.saturating_add(1);
        }
    }
    score
}

/// Ordered list of `(path, content)` to inject, respecting caps and `[skills].files` filter.
pub fn select_skills_for_prompt(
    cwd: &Path,
    app: &AppConfig,
    current_user_line: &str,
) -> Vec<(std::path::PathBuf, String)> {
    if !crate::config::effective_skills_auto_inject(&app.skills) {
        return Vec::new();
    }

    let cfg = &app.skills;
    let max_files = effective_skills_max_files(cfg) as usize;
    let max_chars = effective_skills_max_total_chars(cfg) as usize;

    let mut candidates = list_skill_paths(cwd);
    if !cfg.files.is_empty() {
        let allowed: HashSet<String> = cfg.files.iter().cloned().collect();
        candidates.retain(|p| {
            p.file_name()
                .map(|n| allowed.contains(&n.to_string_lossy().into_owned()))
                .unwrap_or(false)
        });
    }

    let user_tokens = tokenize_lower(current_user_line);
    let mut scored: Vec<(u32, std::path::PathBuf, String)> = Vec::new();
    for path in candidates {
        let Some(text) = read_skill_file(&path) else {
            continue;
        };
        let excerpt = skill_excerpt_for_scoring(&text);
        let s = score_skill(&user_tokens, &path, &excerpt);
        scored.push((s, path, text));
    }

    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    if scored.iter().all(|(s, _, _)| *s == 0) && !scored.is_empty() {
        scored.sort_by(|a, b| a.1.cmp(&b.1));
    }

    let mut out = Vec::new();
    let mut used = 0usize;
    for (_, path, content) in scored.into_iter().take(max_files) {
        let header = format!(
            "--- skill: {} ---\n",
            path.file_name().unwrap_or_default().to_string_lossy()
        );
        let block_len = header.len() + content.len();
        if used + block_len > max_chars {
            let remain = max_chars.saturating_sub(used).saturating_sub(header.len());
            if remain < 80 {
                break;
            }
            let truncated: String = content.chars().take(remain).collect();
            out.push((path, format!("{header}{truncated}\n...[truncated]\n")));
            break;
        }
        used += block_len;
        out.push((path, format!("{header}{content}\n")));
    }

    out
}

pub fn format_rules_block(text: &str) -> String {
    format!("Project rules (.cantrik/rules.md) — always follow:\n\n{text}\n")
}

pub fn format_skills_block(chunks: &[(std::path::PathBuf, String)]) -> String {
    if chunks.is_empty() {
        return String::new();
    }
    let body: String = chunks.iter().map(|(_, s)| s.as_str()).collect();
    format!("Relevant skill files (.cantrik/skills/):\n\n{body}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use std::fs;

    #[test]
    fn select_prefers_matching_name() {
        let dir = std::env::temp_dir().join(format!("cantrik-skills-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".cantrik/skills")).unwrap();
        fs::write(dir.join(".cantrik/skills/rust.md"), "# Rust\nUse clippy.").unwrap();
        fs::write(dir.join(".cantrik/skills/python.md"), "# Python\nUse ruff.").unwrap();

        let mut app = AppConfig::default();
        app.skills.auto_inject = Some(true);
        app.skills.max_files = Some(2);
        app.skills.max_total_chars = Some(10_000);

        let picked = select_skills_for_prompt(&dir, &app, "fix the rust module");
        assert_eq!(picked.len(), 2);
        assert!(
            picked[0]
                .0
                .file_name()
                .unwrap()
                .to_string_lossy()
                .contains("rust")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn files_list_filters() {
        let dir = std::env::temp_dir().join(format!("cantrik-skills-f-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".cantrik/skills")).unwrap();
        fs::write(dir.join(".cantrik/skills/a.md"), "A").unwrap();
        fs::write(dir.join(".cantrik/skills/b.md"), "B").unwrap();

        let mut app = AppConfig::default();
        app.skills.auto_inject = Some(true);
        app.skills.files = vec!["b.md".into()];

        let picked = select_skills_for_prompt(&dir, &app, "x");
        assert_eq!(picked.len(), 1);
        assert!(picked[0].0.ends_with("b.md"));
        let _ = fs::remove_dir_all(&dir);
    }
}
