//! Dependency intel: `cargo tree`, audit wrapper, manifest mention counts.

use std::fs;
use std::path::Path;
use std::process::Command;

use crate::config::IntelligenceConfig;
use crate::config::effective_audit_command;

use super::IntelligenceError;

/// Run `cargo tree -i <crate> -e normal` from `project_root`.
pub fn run_cargo_tree_invert(
    project_root: &Path,
    crate_name: &str,
) -> Result<String, IntelligenceError> {
    let out = Command::new("cargo")
        .current_dir(project_root)
        .args(["tree", "-i", crate_name, "-e", "normal"])
        .output()
        .map_err(|e| IntelligenceError::Cargo(format!("failed to spawn cargo: {e}")))?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    if !out.status.success() {
        return Err(IntelligenceError::Cargo(format!(
            "{}{}",
            stderr.trim(),
            if stdout.is_empty() {
                String::new()
            } else {
                format!("\n{}", stdout.trim())
            }
        )));
    }
    Ok(stdout)
}

/// First N non-empty lines of tree output (for caps / tests).
pub fn parse_cargo_tree_invert_first_lines(tree_stdout: &str, max: usize) -> Vec<String> {
    tree_stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .take(max)
        .map(String::from)
        .collect()
}

fn is_ignored_dir(name: &str) -> bool {
    matches!(name, "target" | ".git" | "node_modules" | ".cantrik")
}

/// Walk for `Cargo.toml` and count lines mentioning `crate_name` (rough usage).
pub fn count_crate_mentions_in_manifests(
    project_root: &Path,
    crate_name: &str,
    max_files: usize,
) -> usize {
    let mut count = 0;
    let mut scanned = 0usize;
    let needle = format!("{crate_name}");
    walk_cargo_tomls(project_root, max_files, &mut scanned, &mut |body| {
        if body.contains(&needle) {
            // Prefer word-ish match: name in dependency table
            for line in body.lines() {
                let t = line.trim();
                if t.starts_with('#') {
                    continue;
                }
                if t.contains(&needle) {
                    count += 1;
                }
            }
        }
    });
    count
}

fn walk_cargo_tomls(dir: &Path, max_files: usize, scanned: &mut usize, f: &mut impl FnMut(&str)) {
    if *scanned >= max_files {
        return;
    }
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    for ent in rd.flatten() {
        if *scanned >= max_files {
            break;
        }
        let p = ent.path();
        let name = ent.file_name().to_string_lossy().to_string();
        if p.is_dir() {
            if is_ignored_dir(&name) {
                continue;
            }
            walk_cargo_tomls(&p, max_files, scanned, f);
            continue;
        }
        if name == "Cargo.toml" {
            if let Ok(body) = fs::read_to_string(&p) {
                *scanned += 1;
                f(&body);
            }
        }
    }
}

pub fn build_why_context(
    project_root: &Path,
    crate_name: &str,
    tree: &str,
    max_manifest_files: usize,
) -> String {
    let mentions = count_crate_mentions_in_manifests(project_root, crate_name, max_manifest_files);
    format!(
        "## cargo tree -i {crate_name}\n{tree}\n\n## rough Cargo.toml line mentions (heuristic): {mentions}\n",
        crate_name = crate_name,
        tree = tree.trim_end(),
        mentions = mentions
    )
}

pub fn build_upgrade_suggestion_prompt(lock_head: &str, tree_one: &str) -> String {
    format!(
        "You are a Rust dependency advisor. The project uses Cargo. Below is the start of Cargo.lock and a shallow `cargo tree` (depth 1). Suggest safe upgrade priorities (patch/minor first), call out possible breaking changes, and recommend `cargo outdated` or manual review. Do NOT assume upgrades were run.\n\n## Cargo.lock (excerpt)\n{lock}\n\n## cargo tree -d 1 (excerpt)\n{tree}\n",
        lock = lock_head.trim_end(),
        tree = tree_one.trim_end()
    )
}

/// Read first `max_bytes` of Cargo.lock if present.
pub fn read_cargo_lock_excerpt(
    project_root: &Path,
    max_bytes: usize,
) -> Result<String, IntelligenceError> {
    let p = project_root.join("Cargo.lock");
    if !p.exists() {
        return Ok("(no Cargo.lock)\n".into());
    }
    let bytes = fs::read(&p)?;
    let slice = if bytes.len() > max_bytes {
        &bytes[..max_bytes]
    } else {
        &bytes[..]
    };
    Ok(String::from_utf8_lossy(slice).to_string())
}

pub fn run_cargo_tree_depth(project_root: &Path, depth: u32) -> Result<String, IntelligenceError> {
    let out = Command::new("cargo")
        .current_dir(project_root)
        .args(["tree", "-e", "normal", "--depth", &depth.to_string()])
        .output()
        .map_err(|e| IntelligenceError::Cargo(format!("cargo tree: {e}")))?;
    if !out.status.success() {
        return Err(IntelligenceError::Cargo(
            String::from_utf8_lossy(&out.stderr).trim().to_string(),
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// Run configured audit command (default `cargo audit`).
pub fn run_cargo_audit(
    project_root: &Path,
    intel: &IntelligenceConfig,
) -> Result<std::process::Output, IntelligenceError> {
    let argv = effective_audit_command(intel);
    if argv.is_empty() {
        return Err(IntelligenceError::Msg("audit_command is empty".into()));
    }
    let (prog, args) = argv.split_first().unwrap();
    let out = Command::new(prog)
        .current_dir(project_root)
        .args(args)
        .output()
        .map_err(|e| {
            IntelligenceError::Cargo(format!(
                "failed to run {}: {e} (install cargo-audit or set [intelligence].audit_command)",
                argv.join(" ")
            ))
        })?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tree_lines_caps() {
        let s = "a v1\n\nb v2\nc v3\n";
        let v = parse_cargo_tree_invert_first_lines(s, 2);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0], "a v1");
    }
}
