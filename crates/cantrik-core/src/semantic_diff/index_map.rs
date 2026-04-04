//! Map file edits to indexed chunks and intra-file callers.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::indexing::{
    CallEdge, IndexError, SourceChunk, chunks_path, graph_path, read_all_source_chunks,
};

/// Load `chunks.jsonl` grouped by POSIX relative path.
pub fn chunks_by_path(
    project_root: &Path,
) -> Result<HashMap<String, Vec<SourceChunk>>, IndexError> {
    let ast_dir = crate::indexing::ast_index_dir(project_root);
    let chunks_path = chunks_path(&ast_dir);
    if !chunks_path.exists() {
        return Ok(HashMap::new());
    }
    let flat = read_all_source_chunks(&chunks_path)?;
    let mut map: HashMap<String, Vec<SourceChunk>> = HashMap::new();
    for c in flat {
        map.entry(c.path.clone()).or_default().push(c);
    }
    for v in map.values_mut() {
        v.sort_by_key(|c| (c.start_byte, c.symbol.clone()));
    }
    Ok(map)
}

/// Load call edges from `graph.json`.
pub fn load_call_edges(project_root: &Path) -> Result<Vec<CallEdge>, IndexError> {
    let ast_dir = crate::indexing::ast_index_dir(project_root);
    let p = graph_path(&ast_dir);
    if !p.exists() {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(&p)?;
    let edges: Vec<CallEdge> = serde_json::from_str(&text)?;
    Ok(edges)
}

fn chunk_end_row_exclusive(chunk: &SourceChunk) -> usize {
    let n = chunk.source.lines().count().max(1);
    chunk.start_row.saturating_add(n)
}

/// Chunks whose line span intersects any changed new line index.
pub fn affected_chunks_for_path(
    chunks: &[SourceChunk],
    changed_new_lines: &[usize],
) -> Vec<SourceChunk> {
    if changed_new_lines.is_empty() {
        return Vec::new();
    }
    let changed: HashSet<usize> = changed_new_lines.iter().copied().collect();
    chunks
        .iter()
        .filter(|c| {
            let end = chunk_end_row_exclusive(c);
            (c.start_row..end).any(|ln| changed.contains(&ln))
        })
        .cloned()
        .collect()
}

/// Edges in the same file where callee is in `symbols`.
pub fn callers_for_symbols(
    path: &str,
    symbols: &HashSet<String>,
    edges: &[CallEdge],
) -> Vec<CallEdge> {
    edges
        .iter()
        .filter(|e| e.path == path && symbols.contains(&e.callee))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn affected_by_line_overlap() {
        let c = SourceChunk {
            path: "src/lib.rs".into(),
            language: "rust".into(),
            symbol: "foo".into(),
            kind: "function".into(),
            start_byte: 0,
            end_byte: 10,
            start_row: 5,
            start_col: 0,
            source: "fn foo() {}".into(),
        };
        let hit = affected_chunks_for_path(&[c], &[5, 6]);
        assert_eq!(hit.len(), 1);
    }
}
