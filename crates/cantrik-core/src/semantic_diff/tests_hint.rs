//! Heuristic test suggestions without coverage data.

use std::collections::HashSet;
use std::path::Path;

use crate::indexing::SourceChunk;

/// Suggest `cargo test` and list index chunks that look like tests near changed modules.
pub fn test_hints_for_changes(
    _project_root: &Path,
    changed_paths: &[String],
    all_chunks: &[SourceChunk],
) -> Vec<String> {
    let mut hints = Vec::new();
    hints.push("Run `cargo test` (or your project test command) after reviewing changes.".into());

    let changed: HashSet<&str> = changed_paths.iter().map(|s| s.as_str()).collect();

    let mut test_files: Vec<&str> = all_chunks
        .iter()
        .filter(|c| {
            let p = c.path.to_ascii_lowercase();
            p.contains("/tests/")
                || p.contains("\\tests\\")
                || c.source.contains("#[test]")
                || c.source.contains("#[tokio::test]")
        })
        .map(|c| c.path.as_str())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    test_files.sort();

    for cp in changed_paths {
        let stem = Path::new(cp)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if stem.is_empty() {
            continue;
        }
        for tf in &test_files {
            if tf.to_ascii_lowercase().contains(&stem.to_ascii_lowercase()) {
                hints.push(format!(
                    "Possible related tests: {tf} (name overlap with {cp})"
                ));
            }
        }
    }

    if changed
        .iter()
        .any(|p| p.contains("/tests/") || p.contains("\\tests\\"))
    {
        hints.push("Test sources were modified — re-run those modules’ tests.".into());
    }

    hints
}
