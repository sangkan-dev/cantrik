use std::path::Path;

use similar::TextDiff;

use super::read::ToolError;

/// Proof that the caller showed a diff and obtained user confirmation.
pub struct WriteApproval(());

impl WriteApproval {
    /// Call only after printing a unified diff and the user explicitly approves the write.
    #[doc(hidden)]
    pub fn user_confirmed_after_reviewing_diff() -> Self {
        Self(())
    }
}

pub fn unified_diff(path: &Path, old_text: &str, new_text: &str) -> String {
    TextDiff::from_lines(old_text, new_text)
        .unified_diff()
        .context_radius(3)
        .header(
            &format!("a/{}", path.display()),
            &format!("b/{}", path.display()),
        )
        .to_string()
}

/// Prepare diff string by reading current file (empty if missing).
pub fn diff_for_new_contents(path: &Path, new_text: &str) -> Result<String, ToolError> {
    let old = if path.exists() {
        std::fs::read_to_string(path)?
    } else {
        String::new()
    };
    Ok(unified_diff(path, &old, new_text))
}

pub fn commit_write(path: &Path, new_text: &str, _: WriteApproval) -> Result<(), ToolError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, new_text)?;
    Ok(())
}
