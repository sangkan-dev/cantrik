//! Pre-write snapshots under `.cantrik/checkpoints/` (PRD §4.5, Sprint 9).

mod meta;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use meta::CHECKPOINT_SCHEMA_VERSION;
pub use meta::{CheckpointFileMeta, CheckpointMeta};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CheckpointError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("checkpoint: {0}")]
    Msg(String),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

/// `.cantrik/checkpoints/` for a project.
pub fn checkpoints_dir(project_root: &Path) -> PathBuf {
    project_root.join(".cantrik").join("checkpoints")
}

fn allocate_seq(cp_root: &Path) -> Result<u32, CheckpointError> {
    fs::create_dir_all(cp_root)?;
    let seq_file = cp_root.join(".seq");
    let mut n = 0u32;
    if seq_file.exists() {
        let s = fs::read_to_string(&seq_file)?;
        n = s.trim().parse().unwrap_or(0);
    }
    n = n.saturating_add(1);
    fs::write(&seq_file, n.to_string())?;
    Ok(n)
}

fn slug_from_rel(rel: &Path) -> String {
    let s = rel.to_string_lossy().replace(['/', '\\'], "-");
    let s: String = s
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '.')
        .collect();
    if s.is_empty() {
        return "write".into();
    }
    s.chars().take(48).collect()
}

/// Copy pre-write state for `abs_target` (absolute path under project). Creates `checkpoint-NNN-slug/`.
pub fn snapshot_before_write(
    project_root: &Path,
    abs_target: &Path,
    task: Option<&str>,
) -> Result<PathBuf, CheckpointError> {
    let root = project_root
        .canonicalize()
        .map_err(|e| CheckpointError::Msg(format!("canonicalize project_root: {e}")))?;
    let abs = abs_target
        .canonicalize()
        .or_else(|_| {
            // New file: canonicalize parent + join file name
            let parent = abs_target
                .parent()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no parent"))?;
            let name = abs_target.file_name().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "path has no file name")
            })?;
            let p = parent.canonicalize()?.join(name);
            Ok::<PathBuf, io::Error>(p)
        })
        .map_err(CheckpointError::Io)?;

    if !abs.starts_with(&root) {
        return Err(CheckpointError::Msg(
            "target path outside project root".into(),
        ));
    }

    let rel = abs
        .strip_prefix(&root)
        .map_err(|_| CheckpointError::Msg("strip_prefix failed".into()))?
        .to_path_buf();

    let had_previous = abs.exists();
    let cp_root = checkpoints_dir(project_root);
    let seq = allocate_seq(&cp_root)?;
    let slug = slug_from_rel(&rel);
    let dir_name = format!("checkpoint-{seq:03}-{slug}");
    let dir = cp_root.join(&dir_name);
    if dir.exists() {
        return Err(CheckpointError::Msg(format!(
            "checkpoint dir already exists: {}",
            dir.display()
        )));
    }
    fs::create_dir_all(dir.join("files"))?;

    let blob_rel = rel.clone();
    let files_meta = vec![CheckpointFileMeta {
        relative_path: rel.to_string_lossy().replace('\\', "/"),
        had_previous,
    }];

    if had_previous {
        let dest = dir.join("files").join(&blob_rel);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&abs, &dest)?;
    }

    let meta = CheckpointMeta {
        schema_version: CHECKPOINT_SCHEMA_VERSION,
        created_at: chrono::Utc::now().to_rfc3339(),
        seq,
        task: task.map(String::from),
        files: files_meta,
    };
    let meta_path = dir.join("meta.json");
    fs::write(&meta_path, serde_json::to_string_pretty(&meta)?)?;
    Ok(dir)
}

/// Sorted oldest-first (by `seq`).
pub fn list_checkpoints(
    project_root: &Path,
) -> Result<Vec<(PathBuf, CheckpointMeta)>, CheckpointError> {
    let cp_root = checkpoints_dir(project_root);
    if !cp_root.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&cp_root)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with("checkpoint-") || name == ".seq" {
            continue;
        }
        let p = entry.path();
        if !p.is_dir() {
            continue;
        }
        let meta_path = p.join("meta.json");
        if !meta_path.is_file() {
            continue;
        }
        let text = fs::read_to_string(&meta_path)?;
        let meta: CheckpointMeta = serde_json::from_str(&text)?;
        out.push((p, meta));
    }
    out.sort_by_key(|(_, m)| m.seq);
    Ok(out)
}

/// Latest checkpoint directory (highest seq).
pub fn latest_checkpoint_dir(project_root: &Path) -> Result<Option<PathBuf>, CheckpointError> {
    let mut v = list_checkpoints(project_root)?;
    Ok(v.pop().map(|(p, _)| p))
}

/// Apply checkpoint at `checkpoint_dir`: restore blobs; remove files that were new (`had_previous: false`).
pub fn apply_checkpoint(project_root: &Path, checkpoint_dir: &Path) -> Result<(), CheckpointError> {
    let root = project_root
        .canonicalize()
        .map_err(|e| CheckpointError::Msg(format!("canonicalize project_root: {e}")))?;
    let text = fs::read_to_string(checkpoint_dir.join("meta.json"))?;
    let meta: CheckpointMeta = serde_json::from_str(&text)?;
    if meta.schema_version != CHECKPOINT_SCHEMA_VERSION {
        return Err(CheckpointError::Msg(format!(
            "unsupported checkpoint schema {}",
            meta.schema_version
        )));
    }

    for f in &meta.files {
        let rel = Path::new(&f.relative_path);
        let target = root.join(rel);
        if !target.starts_with(&root) {
            return Err(CheckpointError::Msg("path traversal in meta".into()));
        }
        if f.had_previous {
            let src = checkpoint_dir.join("files").join(rel);
            if !src.is_file() {
                return Err(CheckpointError::Msg(format!(
                    "missing snapshot blob: {}",
                    src.display()
                )));
            }
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&src, &target)?;
        } else if target.exists() {
            fs::remove_file(&target)?;
        }
    }
    Ok(())
}

/// Find checkpoint dir by numeric id `001` or unique prefix match on folder name.
pub fn resolve_checkpoint_dir(project_root: &Path, id: &str) -> Result<PathBuf, CheckpointError> {
    let id = id.trim();
    if id.is_empty() {
        return Err(CheckpointError::Msg("empty checkpoint id".into()));
    }
    let v = list_checkpoints(project_root)?;
    let matches: Vec<_> = v
        .into_iter()
        .filter(|(p, m)| {
            let name = p.file_name().unwrap_or_default().to_string_lossy();
            id == format!("{:03}", m.seq) || id == m.seq.to_string() || name.contains(id)
        })
        .collect();
    match matches.len() {
        0 => Err(CheckpointError::Msg(format!(
            "no checkpoint matching {id:?}"
        ))),
        1 => Ok(matches[0].0.clone()),
        _ => Err(CheckpointError::Msg(format!(
            "ambiguous id {id:?}; use full checkpoint folder name"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root(name: &str) -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("cantrik-cp-test-{name}-{id}"))
    }

    #[test]
    fn snapshot_restore_roundtrip() {
        let root = temp_root("rt");
        fs::create_dir_all(root.join("src")).expect("mkdir");
        let f = root.join("src/hello.txt");
        fs::write(&f, "v1").expect("write v1");

        let abs = f.canonicalize().expect("canon");
        let cp_dir = snapshot_before_write(&root, &abs, None).expect("snapshot");
        assert!(cp_dir.join("meta.json").is_file());

        fs::write(&f, "v2").expect("write v2");
        apply_checkpoint(&root, &cp_dir).expect("rollback");
        let s = fs::read_to_string(&f).expect("read");
        assert_eq!(s, "v1");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn new_file_rollback_removes() {
        let root = temp_root("new");
        fs::create_dir_all(root.join("src")).expect("mkdir");
        let f = root.join("src/new.txt");
        let cp_dir = snapshot_before_write(&root, &f, None).expect("snapshot");
        fs::write(&f, "created").expect("create");
        assert!(f.is_file());
        apply_checkpoint(&root, &cp_dir).expect("rollback");
        assert!(!f.exists());
        let _ = fs::remove_dir_all(&root);
    }
}
