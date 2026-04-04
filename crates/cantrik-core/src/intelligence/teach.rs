//! Lightweight repo summary for `cantrik teach` (+ optional wiki formatting).

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::indexing::{ast_index_dir, chunks_path, read_all_source_chunks};

use super::IntelligenceError;

fn ignore_dir(name: &str) -> bool {
    matches!(
        name,
        "target" | ".git" | "node_modules" | ".cantrik" | ".idea" | ".vscode"
    )
}

/// Collect directory names (depth 1), README snippets, workspace crate names, optional AST index summary.
pub fn gather_teach_context(
    project_root: &Path,
    max_files: usize,
) -> Result<String, IntelligenceError> {
    let cap_dirs = max_files.min(64).max(8);
    let top_dirs = list_top_level_dirs(project_root, cap_dirs)?;
    let readmes = gather_readme_excerpts(project_root, 3, 1200)?;
    let crates = list_workspace_crate_dirs(project_root, cap_dirs);
    let index_summary = ast_index_summary(project_root, cap_dirs)?;
    gather_teach_context_from_parts(&top_dirs, &readmes, &crates, &index_summary, max_files)
}

pub fn gather_teach_context_from_parts(
    top_dirs: &str,
    readmes: &str,
    crates: &str,
    index_summary: &str,
    max_files: usize,
) -> Result<String, IntelligenceError> {
    let mut s = String::new();
    s.push_str("## Top-level directories\n");
    s.push_str(top_dirs);
    s.push_str("\n\n## README excerpts\n");
    s.push_str(readmes);
    s.push_str("\n\n## Workspace crates (heuristic)\n");
    s.push_str(crates);
    s.push_str("\n\n## AST index summary (if present)\n");
    s.push_str(index_summary);
    s.push_str(&format!(
        "\n\n(config note: teach_max_files_scanned effective cap used for tree walk: {})\n",
        max_files
    ));
    Ok(s)
}

fn list_top_level_dirs(project_root: &Path, cap: usize) -> Result<String, IntelligenceError> {
    let mut names = Vec::new();
    let rd = fs::read_dir(project_root).map_err(IntelligenceError::Io)?;
    for ent in rd.flatten() {
        if !ent.path().is_dir() {
            continue;
        }
        let n = ent.file_name().to_string_lossy().to_string();
        if ignore_dir(&n) {
            continue;
        }
        names.push(n);
        if names.len() >= cap {
            break;
        }
    }
    names.sort();
    Ok(names.join("\n"))
}

fn gather_readme_excerpts(
    project_root: &Path,
    max_readmes: usize,
    max_chars_each: usize,
) -> Result<String, IntelligenceError> {
    let mut found: Vec<(String, String)> = Vec::new();
    push_readmes_in_dir(project_root, project_root, &mut found, max_readmes)?;
    let crates_dir = project_root.join("crates");
    if crates_dir.is_dir() {
        let rd = fs::read_dir(&crates_dir).map_err(IntelligenceError::Io)?;
        for ent in rd.flatten() {
            if found.len() >= max_readmes {
                break;
            }
            let p = ent.path();
            if !p.is_dir() {
                continue;
            }
            let name = ent.file_name().to_string_lossy().to_string();
            if ignore_dir(&name) {
                continue;
            }
            push_readmes_in_dir(project_root, &p, &mut found, max_readmes)?;
        }
    }
    let mut out = String::new();
    for (path, body) in found {
        out.push_str(&format!("### {}\n", path));
        let excerpt: String = body.chars().take(max_chars_each).collect();
        out.push_str(&excerpt);
        if body.chars().count() > max_chars_each {
            out.push_str("\n…\n");
        } else {
            out.push('\n');
        }
    }
    if out.is_empty() {
        out.push_str("(no README*.md at repo root or crates/*/)\n");
    }
    Ok(out)
}

fn push_readmes_in_dir(
    project_root: &Path,
    dir: &Path,
    out: &mut Vec<(String, String)>,
    max: usize,
) -> Result<(), IntelligenceError> {
    if out.len() >= max {
        return Ok(());
    }
    let rd = fs::read_dir(dir).map_err(IntelligenceError::Io)?;
    for ent in rd.flatten() {
        if out.len() >= max {
            break;
        }
        let p = ent.path();
        if !p.is_file() {
            continue;
        }
        let name = ent.file_name().to_string_lossy().to_string();
        let low = name.to_ascii_lowercase();
        if low.starts_with("readme") && low.ends_with(".md") {
            let rel = p
                .strip_prefix(project_root)
                .unwrap_or(&p)
                .display()
                .to_string();
            let body = fs::read_to_string(&p).map_err(IntelligenceError::Io)?;
            out.push((rel, body));
        }
    }
    Ok(())
}

fn list_workspace_crate_dirs(project_root: &Path, cap: usize) -> String {
    let crates = project_root.join("crates");
    if !crates.is_dir() {
        return "(no crates/ directory)\n".into();
    }
    let mut names = Vec::new();
    let Ok(rd) = fs::read_dir(&crates) else {
        return "(could not read crates/)\n".into();
    };
    for ent in rd.flatten() {
        let p = ent.path();
        if !p.is_dir() {
            continue;
        }
        let n = ent.file_name().to_string_lossy().to_string();
        if p.join("Cargo.toml").is_file() {
            names.push(n);
        }
        if names.len() >= cap {
            break;
        }
    }
    names.sort();
    names.join("\n")
}

fn ast_index_summary(project_root: &Path, max_symbols: usize) -> Result<String, IntelligenceError> {
    let ast = ast_index_dir(project_root);
    let cp = chunks_path(&ast);
    if !cp.exists() {
        return Ok("(no `.cantrik/index/ast/chunks.jsonl`; run `cantrik index`)\n".into());
    }
    let chunks = read_all_source_chunks(&cp).map_err(|e| IntelligenceError::Msg(e.to_string()))?;
    let mut by_path: HashMap<String, Vec<String>> = HashMap::new();
    for c in chunks {
        by_path
            .entry(c.path)
            .or_default()
            .push(format!("{} ({})", c.symbol, c.kind));
    }
    let mut paths: Vec<_> = by_path.keys().cloned().collect();
    paths.sort();
    let mut lines = Vec::new();
    let mut n = 0usize;
    for path in paths {
        if n >= max_symbols {
            lines.push("… (truncated)".into());
            break;
        }
        let syms = &by_path[&path];
        let head: Vec<_> = syms.iter().take(5).cloned().collect();
        lines.push(format!("{}: {}", path, head.join(", ")));
        n += 1;
    }
    Ok(lines.join("\n"))
}

pub fn build_teach_prompt(context: &str, wiki: bool) -> String {
    let wiki_note = if wiki {
        "Use Markdown with YAML frontmatter (title, tags as a list). Use wikilinks like [[ModuleName]] for major sections. Obsidian-friendly.\n"
    } else {
        ""
    };
    format!(
        "You are a technical writer. From the repository context below, produce:\n\
1) `ARCHITECTURE.md` content (overview, main modules, data flow at a high level).\n\
2) 1–3 ADR stubs (title + context + decision + status placeholder).\n\
3) A short \"Public API / entry points\" section.\n\
{wiki_note}\
Do not invent files that contradict the tree; mark uncertainty explicitly.\n\n---\n{context}\n---",
        wiki_note = wiki_note,
        context = context
    )
}

/// Wrap generated body with minimal YAML frontmatter and encourage `[[wikilinks]]` in headings.
pub fn apply_wiki_format(title: &str, body: &str) -> String {
    format!(
        "---\ntitle: {title}\ntags: [cantrik-teach, architecture]\n---\n\n# [[{title}]]\n\n{body}",
        title = title,
        body = body
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gather_from_parts_includes_sections() {
        let s = gather_teach_context_from_parts("a\nb", "r", "c1", "idx", 8).unwrap();
        assert!(s.contains("Top-level"));
        assert!(s.contains("README"));
        assert!(s.contains("Workspace crates"));
        assert!(s.contains("AST index"));
    }

    #[test]
    fn wiki_format_has_frontmatter() {
        let w = apply_wiki_format("MyApp", "hello");
        assert!(w.starts_with("---"));
        assert!(w.contains("[[MyApp]]"));
    }
}
