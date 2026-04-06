use std::path::Path;

use similar::{ChangeTag, TextDiff};

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
/// Approximate inserted / deleted **line** counts between two file bodies (for REPL summaries).
pub fn line_insert_delete_counts(old_text: &str, new_text: &str) -> (usize, usize) {
    let diff = TextDiff::from_lines(old_text, new_text);
    let mut ins = 0usize;
    let mut del = 0usize;
    for change in diff.iter_all_changes() {
        let v = change.value();
        if v.is_empty() {
            continue;
        }
        let n = {
            let c = v.lines().count();
            if v.ends_with('\n') { c } else { c.max(1) }
        };
        match change.tag() {
            ChangeTag::Insert => ins += n,
            ChangeTag::Delete => del += n,
            ChangeTag::Equal => {}
        }
    }
    (ins, del)
}

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

#[cfg(test)]
mod line_count_tests {
    use super::line_insert_delete_counts;

    #[test]
    fn counts_ins_and_dels() {
        let (i, d) = line_insert_delete_counts("a\nb\n", "a\nc\n");
        assert!(i >= 1 && d >= 1, "i={i} d={d}");
    }
}
