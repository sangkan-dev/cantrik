//! Codebase AST indexing (Sprint 5): scan, tree-sitter chunking, intra-file call graph, artifacts under `.cantrik/index/ast/`.

mod chunk;
mod graph;
mod lsp_symbols;
mod manifest;
mod scan;
mod writer;

use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use lsp_symbols::{
    IndexedSymbolOutline, load_chunks_for_document, outlines_from_chunks, parse_chunks_jsonl,
};
pub use manifest::Manifest;
pub use writer::{chunks_path, graph_path, read_all_source_chunks};

/// Default cap per file before skipping (1 MiB).
pub const DEFAULT_MAX_FILE_BYTES: u64 = 1024 * 1024;

/// Artifacts live under `<project_root>/.cantrik/index/ast/` so Sprint 6 can own `.cantrik/index/` for LanceDB.
pub fn ast_index_dir(project_root: &Path) -> std::path::PathBuf {
    project_root.join(".cantrik").join("index").join("ast")
}

#[derive(Debug, Clone)]
pub struct IndexOptions {
    pub max_file_bytes: u64,
}

impl Default for IndexOptions {
    fn default() -> Self {
        Self {
            max_file_bytes: DEFAULT_MAX_FILE_BYTES,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceChunk {
    /// Path relative to project root, POSIX separators where possible.
    pub path: String,
    pub language: String,
    pub symbol: String,
    pub kind: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_row: usize,
    pub start_col: usize,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallEdge {
    pub path: String,
    pub caller: String,
    pub callee: String,
    pub line: usize,
}

#[derive(Debug, Default)]
pub struct BuildReport {
    pub files_scanned: usize,
    pub files_indexed: usize,
    pub files_skipped_unsupported: usize,
    pub files_skipped_binary: usize,
    pub files_skipped_size: usize,
    pub files_reused: usize,
    pub chunks: usize,
    pub edges: usize,
}

#[derive(Debug, Error)]
pub enum IndexError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("index internal error: {0}")]
    Internal(String),
}

/// Run full index pass at `project_root` (directory being indexed). Writes `.cantrik/index/ast/*`.
pub fn build_index(project_root: &Path, opts: &IndexOptions) -> Result<BuildReport, IndexError> {
    let root = std::fs::canonicalize(project_root).unwrap_or_else(|_| project_root.to_path_buf());

    let scanned = scan::scan_repo_files(&root, opts)?;
    let ast_dir = ast_index_dir(&root);
    let manifest_path = writer::manifest_path(&ast_dir);
    let chunks_path = writer::chunks_path(&ast_dir);

    let prev_manifest = Manifest::load(&manifest_path)?;
    let mut prev_chunks = writer::read_chunks_grouped(&chunks_path)?;

    let mut report = BuildReport {
        files_scanned: scanned.len(),
        ..Default::default()
    };

    let mut new_manifest = Manifest::default();
    let mut all_chunks_by_path: std::collections::BTreeMap<String, Vec<SourceChunk>> =
        std::collections::BTreeMap::new();

    for file in &scanned {
        let rel = file
            .strip_prefix(&root)
            .map_err(|_| IndexError::Internal("file path outside index root".into()))?;
        let rel_str = rel.to_string_lossy().replace('\\', "/");

        let meta = std::fs::metadata(file)?;
        if meta.len() > opts.max_file_bytes {
            report.files_skipped_size += 1;
            continue;
        }

        let bytes = std::fs::read(file)?;

        if scan::is_probably_binary(&bytes) {
            report.files_skipped_binary += 1;
            continue;
        }

        let hash = manifest::hash_bytes(&bytes);
        let reuse = prev_manifest
            .files
            .get(&rel_str)
            .map(|h| h == &hash)
            .unwrap_or(false)
            && prev_chunks.contains_key(&rel_str);

        if reuse {
            new_manifest.files.insert(rel_str.clone(), hash);
            let chunks = prev_chunks.remove(&rel_str).unwrap_or_default();
            report.chunks += chunks.len();
            report.files_reused += 1;
            all_chunks_by_path.insert(rel_str, chunks);
            continue;
        }

        let (_lang, chunks) = match chunk::extract_chunks(&rel_str, &bytes) {
            Some((_lang, c)) if !c.is_empty() => (_lang, c),
            Some(_) => {
                report.files_skipped_unsupported += 1;
                continue;
            }
            None => {
                report.files_skipped_unsupported += 1;
                continue;
            }
        };

        new_manifest.files.insert(rel_str.clone(), hash);
        report.files_indexed += 1;
        report.chunks += chunks.len();
        all_chunks_by_path.insert(rel_str, chunks);
    }

    let mut flat_chunks: Vec<SourceChunk> =
        all_chunks_by_path.values().flatten().cloned().collect();
    flat_chunks.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then_with(|| a.start_byte.cmp(&b.start_byte))
    });

    let edges = graph::extract_edges(&root, &flat_chunks);

    report.edges = edges.len();

    writer::write_all(&ast_dir, &new_manifest, &flat_chunks, &edges)?;

    Ok(report)
}
