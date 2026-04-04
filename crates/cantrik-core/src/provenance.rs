//! Optional `.cantrik/provenance.jsonl` (PRD §4.10, Sprint 9).

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::Serialize;

const ENV_TASK: &str = "CANTRIK_TASK";

#[derive(Debug, Serialize)]
pub struct ProvenanceRecord {
    pub path: String,
    pub timestamp: String,
    pub model: Option<String>,
    pub task: Option<String>,
}

pub fn provenance_jsonl_path(project_root: &Path) -> PathBuf {
    project_root.join(".cantrik").join("provenance.jsonl")
}

pub fn append_provenance_record(
    project_root: &Path,
    rel_path: &str,
    model: Option<String>,
) -> std::io::Result<()> {
    let rec = ProvenanceRecord {
        path: rel_path.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        model,
        task: std::env::var(ENV_TASK).ok(),
    };
    let p = provenance_jsonl_path(project_root);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut f = OpenOptions::new().create(true).append(true).open(p)?;
    writeln!(f, "{}", serde_json::to_string(&rec).map_err(to_io)?)?;
    f.sync_all()?;
    Ok(())
}

fn to_io(e: serde_json::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, e)
}
