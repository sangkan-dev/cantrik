use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::{CallEdge, IndexError, Manifest, SourceChunk};

pub(crate) fn manifest_path(ast_dir: &Path) -> PathBuf {
    ast_dir.join("manifest.json")
}

pub fn chunks_path(ast_dir: &Path) -> PathBuf {
    ast_dir.join("chunks.jsonl")
}

pub(crate) fn graph_path(ast_dir: &Path) -> PathBuf {
    ast_dir.join("graph.json")
}

pub(crate) fn write_all(
    ast_dir: &Path,
    manifest: &Manifest,
    chunks: &[SourceChunk],
    edges: &[CallEdge],
) -> Result<(), IndexError> {
    std::fs::create_dir_all(ast_dir)?;
    manifest.save(&manifest_path(ast_dir))?;

    let chunks_p = chunks_path(ast_dir);
    let mut buf = String::new();
    for c in chunks {
        buf.push_str(&serde_json::to_string(c)?);
        buf.push('\n');
    }
    std::fs::write(&chunks_p, buf)?;

    let graph_p = graph_path(ast_dir);
    std::fs::write(&graph_p, serde_json::to_string_pretty(edges)?)?;

    Ok(())
}

/// All `SourceChunk` lines from `chunks.jsonl` in file order.
pub fn read_all_source_chunks(path: &Path) -> Result<Vec<super::SourceChunk>, IndexError> {
    let mut out = Vec::new();
    if !path.exists() {
        return Ok(out);
    }
    let text = std::fs::read_to_string(path)?;
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        out.push(serde_json::from_str(line)?);
    }
    Ok(out)
}

pub(crate) fn read_chunks_grouped(
    path: &Path,
) -> Result<HashMap<String, Vec<SourceChunk>>, IndexError> {
    let mut map: HashMap<String, Vec<SourceChunk>> = HashMap::new();
    if !path.exists() {
        return Ok(map);
    }
    let text = std::fs::read_to_string(path)?;
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let chunk: SourceChunk = serde_json::from_str(line)?;
        map.entry(chunk.path.clone()).or_default().push(chunk);
    }
    Ok(map)
}
