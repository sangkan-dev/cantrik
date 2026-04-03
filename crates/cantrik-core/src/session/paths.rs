//! XDG data dir: `~/.local/share/cantrik/memory.db` (overridable).

use std::path::{Path, PathBuf};

/// Override for tests: absolute path to SQLite file.
pub const ENV_MEMORY_DB: &str = "CANTRIK_MEMORY_DB";

pub fn share_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cantrik")
}

pub fn memory_db_path() -> PathBuf {
    if let Ok(p) = std::env::var(ENV_MEMORY_DB) {
        let pb = PathBuf::from(p);
        if let Some(parent) = pb.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        return pb;
    }
    let dir = share_dir();
    let _ = std::fs::create_dir_all(&dir);
    dir.join("memory.db")
}

pub fn global_anchors_path() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    home.join(".config").join("cantrik").join("anchors.md")
}

pub fn project_anchors_path(project_root: &Path) -> PathBuf {
    project_root.join(".cantrik").join("anchors.md")
}
