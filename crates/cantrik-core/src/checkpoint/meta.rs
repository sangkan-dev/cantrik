use serde::{Deserialize, Serialize};

pub const CHECKPOINT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckpointFileMeta {
    pub relative_path: String,
    pub had_previous: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckpointMeta {
    pub schema_version: u32,
    pub created_at: String,
    pub seq: u32,
    pub task: Option<String>,
    pub files: Vec<CheckpointFileMeta>,
}
