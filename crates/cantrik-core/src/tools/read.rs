use std::path::Path;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("file too large ({size} bytes > max {max})")]
    TooLarge { size: u64, max: u64 },
    #[error("{0}")]
    Msg(String),
}

pub fn read_file_capped(path: &Path, max_bytes: u64) -> Result<String, ToolError> {
    let meta = std::fs::metadata(path)?;
    if meta.len() > max_bytes {
        return Err(ToolError::TooLarge {
            size: meta.len(),
            max: max_bytes,
        });
    }
    let bytes = std::fs::read(path)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}
