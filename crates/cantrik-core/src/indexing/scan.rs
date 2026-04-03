use std::path::{Path, PathBuf};

use ignore::WalkBuilder;

use super::IndexOptions;

pub(crate) fn scan_repo_files(
    root: &Path,
    opts: &IndexOptions,
) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut out = Vec::new();
    let mut wb = WalkBuilder::new(root);
    wb.git_ignore(true);
    wb.git_global(true);
    wb.git_exclude(true);
    wb.hidden(false);
    wb.standard_filters(true);

    for entry in wb.build() {
        let entry = entry.map_err(std::io::Error::other)?;
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let path = entry.path();
        if let Ok(meta) = path.metadata()
            && meta.len() > opts.max_file_bytes
        {
            continue;
        }
        if is_supported_extension(path) {
            out.push(path.to_path_buf());
        }
    }

    out.sort();
    Ok(out)
}

fn is_supported_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            matches!(
                e.to_ascii_lowercase().as_str(),
                "rs" | "py"
                    | "js"
                    | "mjs"
                    | "cjs"
                    | "ts"
                    | "mts"
                    | "cts"
                    | "tsx"
                    | "go"
                    | "java"
                    | "c"
                    | "h"
                    | "cc"
                    | "cpp"
                    | "cxx"
                    | "hpp"
                    | "hxx"
                    | "php"
                    | "rb"
                    | "sql"
                    | "toml"
                    | "json"
                    | "yaml"
                    | "yml"
                    | "md"
                    | "markdown"
            )
        })
        .unwrap_or(false)
}

/// Heuristic: NUL byte in first window, or very high non-printable ratio.
pub(crate) fn is_probably_binary(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    let window = bytes.len().min(8192);
    let sample = &bytes[..window];
    if sample.contains(&0) {
        return true;
    }
    let non_text = sample
        .iter()
        .copied()
        .filter(|&b| b != b'\n' && b != b'\r' && b != b'\t' && (b < 32 || b == 127))
        .count();
    non_text * 10 > sample.len() * 3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_nul_detected() {
        assert!(is_probably_binary(&[b'h', b'i', 0, b'x']));
    }

    #[test]
    fn utf8_text_not_binary() {
        assert!(!is_probably_binary("fn main() {}\n".as_bytes()));
    }
}
