//! Build semantic diff report: text diff + index-backed symbols and callers.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::tools::unified_diff;

use super::diff_lines::changed_new_line_indices;
use super::git_workspace::{GitWorkspaceError, changed_paths, show_blob};
use super::index_map::{
    affected_chunks_for_path, callers_for_symbols, chunks_by_path, load_call_edges,
};
use super::risk::{RiskLevel, assess_risk};
use super::tests_hint::test_hints_for_changes;

#[derive(Debug, Clone)]
pub struct FileSemanticEntry {
    pub path: String,
    pub unified_diff: String,
    pub affected_symbols: Vec<String>,
    pub callers: Vec<String>,
}

#[derive(Debug)]
pub struct SemanticReport {
    pub index_present: bool,
    pub files: Vec<FileSemanticEntry>,
    pub risk: RiskLevel,
    pub risk_notes: Vec<String>,
    pub test_hints: Vec<String>,
    pub approx_loc_added: usize,
}

fn read_worktree_file(root: &Path, rel: &str) -> String {
    let p = root.join(rel);
    std::fs::read_to_string(&p).unwrap_or_default()
}

fn count_plus_lines(diff: &str) -> usize {
    diff.lines().filter(|l| l.starts_with('+')).count()
}

/// Build report for changed files vs `HEAD` (or empty for new files).
pub fn build_semantic_report(
    project_root: &Path,
    staged: bool,
    include_semantic_from_index: bool,
    max_files: Option<usize>,
) -> Result<SemanticReport, GitWorkspaceError> {
    let mut paths = changed_paths(project_root, staged)?;
    if let Some(cap) = max_files.filter(|n| *n > 0) {
        if paths.len() > cap {
            paths.truncate(cap);
        }
    }
    let chunks_map = chunks_by_path(project_root).unwrap_or_default();
    let index_present = !chunks_map.is_empty();
    let edges = load_call_edges(project_root).unwrap_or_default();
    let all_chunks_flat: Vec<_> = chunks_map.values().flatten().cloned().collect();

    let mut files_out = Vec::new();
    let mut combined_diff = String::new();
    let mut approx_loc_added = 0usize;

    for rel in &paths {
        let old = show_blob(project_root, &format!("HEAD:{rel}"))
            .ok()
            .flatten()
            .unwrap_or_default();
        let new = read_worktree_file(project_root, rel);
        let diff = unified_diff(&PathBuf::from(rel), &old, &new);
        approx_loc_added += count_plus_lines(&diff);
        combined_diff.push_str(&diff);
        combined_diff.push('\n');

        let mut affected_symbols = Vec::new();
        let mut callers = Vec::new();

        if include_semantic_from_index && index_present {
            if let Some(chunks) = chunks_map.get(rel) {
                let lines = changed_new_line_indices(&old, &new);
                let affected = affected_chunks_for_path(chunks, &lines);
                let sym: HashSet<String> = affected.iter().map(|c| c.symbol.clone()).collect();
                affected_symbols = sym.iter().cloned().collect();
                affected_symbols.sort();
                let ce = callers_for_symbols(rel, &sym, &edges);
                callers = ce
                    .iter()
                    .map(|e| format!("{} (line {}) calls {}", e.caller, e.line, e.callee))
                    .collect();
                callers.sort();
            }
        }

        files_out.push(FileSemanticEntry {
            path: rel.clone(),
            unified_diff: diff,
            affected_symbols,
            callers,
        });
    }

    let path_strings: Vec<String> = paths.clone();
    let (risk, risk_notes) =
        assess_risk(&path_strings, &combined_diff, paths.len(), approx_loc_added);
    let test_hints = test_hints_for_changes(project_root, &path_strings, &all_chunks_flat);

    Ok(SemanticReport {
        index_present,
        files: files_out,
        risk,
        risk_notes,
        test_hints,
        approx_loc_added,
    })
}
