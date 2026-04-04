//! Map AST index chunks to editor symbol outlines (Sprint 18 LSP MVP).

use std::path::Path;

use super::{ast_index_dir, chunks_path, read_all_source_chunks, IndexError, SourceChunk};

/// Symbol span derived from indexed chunks (no LSP types in core).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedSymbolOutline {
    pub name: String,
    pub kind: String,
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
}

/// Load chunks for `document_abs` from `{project_root}/.cantrik/index/ast/chunks.jsonl`.
/// Returns empty if the index file is missing or unreadable paths are skipped.
pub fn load_chunks_for_document(
    project_root: &Path,
    document_abs: &Path,
) -> Result<Vec<SourceChunk>, IndexError> {
    let cp = chunks_path(&ast_index_dir(project_root));
    if !cp.exists() {
        return Ok(Vec::new());
    }
    let chunks = read_all_source_chunks(&cp)?;
    let doc = std::fs::canonicalize(document_abs).unwrap_or_else(|_| document_abs.to_path_buf());
    let root = std::fs::canonicalize(project_root).unwrap_or_else(|_| project_root.to_path_buf());
    Ok(chunks
        .into_iter()
        .filter(|c| {
            let joined = root.join(&c.path);
            match std::fs::canonicalize(&joined) {
                Ok(p) => p == doc,
                Err(_) => false,
            }
        })
        .collect())
}

pub fn outlines_from_chunks(chunks: &[SourceChunk]) -> Vec<IndexedSymbolOutline> {
    chunks.iter().map(|c| outline_from_chunk(c)).collect()
}

fn outline_from_chunk(c: &SourceChunk) -> IndexedSymbolOutline {
    let (end_row, end_col) = end_position_for_chunk(c);
    IndexedSymbolOutline {
        name: c.symbol.clone(),
        kind: c.kind.clone(),
        start_row: c.start_row as u32,
        start_col: c.start_col as u32,
        end_row,
        end_col,
    }
}

fn end_position_for_chunk(c: &SourceChunk) -> (u32, u32) {
    let lines: Vec<&str> = if c.source.is_empty() {
        vec![""]
    } else {
        c.source.lines().collect()
    };
    let n = lines.len().max(1);
    let end_line = (c.start_row + n - 1) as u32;
    let last_len = lines.last().map(|s| s.chars().count() as u32).unwrap_or(0);
    let end_col = if n <= 1 {
        (c.start_col as u32).saturating_add(last_len)
    } else {
        last_len
    };
    (end_line, end_col)
}

/// Parse JSONL chunk lines (for tests and tooling).
pub fn parse_chunks_jsonl(text: &str) -> Result<Vec<SourceChunk>, IndexError> {
    let mut out = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        out.push(serde_json::from_str(line)?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jsonl_to_outlines_maps_kind_and_range() {
        let jsonl = r#"{"path":"src/lib.rs","language":"rust","symbol":"foo","kind":"function","start_byte":0,"end_byte":1,"start_row":2,"start_col":4,"source":"fn foo() {\n  1\n}"}"#;
        let chunks = parse_chunks_jsonl(jsonl).expect("parse");
        let o = outlines_from_chunks(&chunks);
        assert_eq!(o.len(), 1);
        assert_eq!(o[0].name, "foo");
        assert_eq!(o[0].kind, "function");
        assert_eq!(o[0].start_row, 2);
        assert_eq!(o[0].start_col, 4);
        assert!(o[0].end_row >= o[0].start_row);
    }
}
