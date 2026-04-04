//! Build Mermaid text for `cantrik visualize` and REPL `/visualize`.

use std::collections::BTreeSet;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::process::Command;

use serde_json;

use crate::indexing::{CallEdge, ast_index_dir, graph_path};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum VisualizeError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Msg(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisualizeKind {
    Callgraph,
    Architecture,
    Dependencies,
}

fn ignore_dir(name: &str) -> bool {
    matches!(
        name,
        "target" | ".git" | "node_modules" | ".cantrik" | ".idea" | ".vscode"
    )
}

fn stable_id(s: &str) -> String {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    format!("n{:x}", h.finish())
}

fn escape_mermaid_label(s: &str) -> String {
    s.replace('"', "'").replace('\n', " ")
}

/// Load intra-file call edges from `.cantrik/index/ast/graph.json`.
pub fn load_call_edges(project_root: &Path) -> Result<Vec<CallEdge>, VisualizeError> {
    let p = graph_path(&ast_index_dir(project_root));
    if !p.exists() {
        return Err(VisualizeError::Msg(format!(
            "no AST graph at {}; run `cantrik index` first",
            p.display()
        )));
    }
    let text = fs::read_to_string(&p)?;
    let edges: Vec<CallEdge> = serde_json::from_str(&text)?;
    Ok(edges)
}

pub fn mermaid_callgraph(edges: &[CallEdge]) -> String {
    if edges.is_empty() {
        return "# No call edges in index.\n".to_string();
    }
    let mut out = String::from("```mermaid\nflowchart LR\n");
    for e in edges {
        let la = format!("{}:{} → {}", e.path, e.caller, e.callee);
        let id_from = stable_id(&format!("{}|{}|{}", e.path, e.caller, e.line));
        let id_to = stable_id(&format!("{}|{}", e.path, e.callee));
        let lf = escape_mermaid_label(&la);
        let lt = escape_mermaid_label(&format!("{}::{}", e.path, e.callee));
        out.push_str(&format!("  {id_from}[\"{lf}\"] --> {id_to}[\"{lt}\"]\n"));
    }
    out.push_str("```\n");
    out
}

pub fn mermaid_architecture_top_dirs(project_root: &Path) -> Result<String, VisualizeError> {
    let mut names = Vec::new();
    for ent in fs::read_dir(project_root).map_err(VisualizeError::Io)? {
        let ent = ent.map_err(VisualizeError::Io)?;
        if !ent.path().is_dir() {
            continue;
        }
        let n = ent.file_name().to_string_lossy().to_string();
        if ignore_dir(&n) {
            continue;
        }
        names.push(n);
    }
    names.sort();
    let mut out = String::from("```mermaid\nflowchart TD\n  root[\"project root\"]\n");
    for n in &names {
        let id = stable_id(n);
        let label = escape_mermaid_label(n);
        out.push_str(&format!("  root --> {id}[\"{label}\"]\n"));
    }
    out.push_str("```\n");
    Ok(out)
}

pub fn mermaid_dependencies_from_cargo_tree_stdout(tree: &str) -> String {
    let mut order = Vec::new();
    let mut set = BTreeSet::new();
    for line in tree.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        let rest = t.trim_start_matches(['│', '├', '└', '─', ' ']);
        let token = rest.split_whitespace().next().unwrap_or("");
        if token.is_empty() || token == "(*)" {
            continue;
        }
        if token
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            && set.insert(token.to_string())
        {
            order.push(token.to_string());
        }
    }
    if order.is_empty() {
        return "# No crate names parsed from cargo tree output.\n".to_string();
    }
    let mut out = String::from("```mermaid\nflowchart TD\n");
    out.push_str("  legend[\"dependency names from cargo tree (MVP; edges omitted)\"]\n");
    for s in order {
        let id = stable_id(&s);
        let label = escape_mermaid_label(&s);
        out.push_str(&format!("  {id}[\"{label}\"]\n"));
    }
    out.push_str("```\n");
    out
}

fn run_cargo_tree(root: &Path) -> Result<String, VisualizeError> {
    let out = Command::new("cargo")
        .current_dir(root)
        .args(["tree", "-e", "normal", "--depth", "3"])
        .output()
        .map_err(|e| VisualizeError::Msg(format!("cargo tree: {e}")))?;
    if !out.status.success() {
        return Err(VisualizeError::Msg(
            String::from_utf8_lossy(&out.stderr).trim().to_string(),
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

pub fn render_visualize_kind(
    kind: VisualizeKind,
    project_root: &Path,
) -> Result<String, VisualizeError> {
    match kind {
        VisualizeKind::Callgraph => {
            let edges = load_call_edges(project_root)?;
            Ok(mermaid_callgraph(&edges))
        }
        VisualizeKind::Architecture => mermaid_architecture_top_dirs(project_root),
        VisualizeKind::Dependencies => {
            let tree = run_cargo_tree(project_root)?;
            Ok(mermaid_dependencies_from_cargo_tree_stdout(&tree))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexing::CallEdge;

    #[test]
    fn callgraph_mermaid_wraps() {
        let edges = vec![CallEdge {
            path: "a.rs".into(),
            caller: "foo".into(),
            callee: "bar".into(),
            line: 1,
        }];
        let m = mermaid_callgraph(&edges);
        assert!(m.contains("```mermaid"));
        assert!(m.contains("foo"));
    }

    #[test]
    fn deps_from_sample_tree() {
        let s = "foo v1\n├── bar v2\n";
        let m = mermaid_dependencies_from_cargo_tree_stdout(s);
        assert!(m.contains("foo"));
        assert!(m.contains("bar"));
    }
}
