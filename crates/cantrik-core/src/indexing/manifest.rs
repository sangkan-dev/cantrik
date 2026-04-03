use std::collections::HashMap;
use std::path::Path;

use hex::encode as hex_encode;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// schema_version for future migrations
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    /// relative_path -> sha256 hex of raw file bytes
    #[serde(default)]
    pub files: HashMap<String, String>,
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            schema_version: Self::CURRENT_VERSION,
            files: HashMap::new(),
        }
    }
}

fn default_schema_version() -> u32 {
    1
}

impl Manifest {
    pub const CURRENT_VERSION: u32 = 1;

    pub fn load(path: &Path) -> Result<Self, std::io::Error> {
        if !path.exists() {
            return Ok(Self {
                schema_version: Self::CURRENT_VERSION,
                files: HashMap::new(),
            });
        }
        let text = std::fs::read_to_string(path)?;
        let m: Manifest = serde_json::from_str(&text).unwrap_or_else(|_| Self {
            schema_version: Self::CURRENT_VERSION,
            files: HashMap::new(),
        });
        Ok(m)
    }

    pub fn save(&self, path: &Path) -> Result<(), std::io::Error> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let text = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(path, text)
    }
}

pub(crate) fn hash_bytes(bytes: &[u8]) -> String {
    let d = Sha256::digest(bytes);
    hex_encode(d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_stable_for_same_input() {
        let h = hash_bytes(b"snapshot");
        assert_eq!(h, hash_bytes(b"snapshot"));
        assert_ne!(h, hash_bytes(b"snapsh0t"));
    }
}
