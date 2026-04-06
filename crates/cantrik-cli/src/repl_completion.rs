//! REPL `@` path and `#` command completion (ratatui list + filter).

use std::path::Path;

use ignore::WalkBuilder;

/// Cap how many paths we collect (large repos).
pub const MAX_PATH_SCAN: usize = 4000;
/// Visible rows in the completion popup.
pub const MAX_VISIBLE: usize = 36;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    Path,
    Hash,
}

#[derive(Debug, Clone)]
pub struct ReplCompletion {
    pub kind: CompletionKind,
    /// Byte index of `@` or `#` in `input`.
    pub trigger_pos: usize,
    pub all_items: Vec<String>,
    pub filtered: Vec<String>,
    pub selected: usize,
}

impl ReplCompletion {
    pub fn refresh_from_input(&mut self, input: &str) {
        let q = query_after_trigger(input, self.trigger_pos);
        let src: &[String] = match self.kind {
            CompletionKind::Path => &self.all_items,
            CompletionKind::Hash => &self.all_items,
        };
        self.filtered = filter_candidates(src, q, MAX_VISIBLE);
        if self.filtered.is_empty() {
            self.selected = 0;
        } else {
            self.selected = self.selected.min(self.filtered.len() - 1);
        }
    }

    /// Replace the fragment after the trigger with the selected item (drops old query).
    pub fn apply_to_input(&self, input: &mut String) -> bool {
        if self.filtered.is_empty() {
            return false;
        }
        let pick = &self.filtered[self.selected];
        if input.len() < self.trigger_pos.saturating_add(1) {
            return false;
        }
        input.truncate(self.trigger_pos.saturating_add(1));
        input.push_str(pick);
        true
    }
}

fn query_after_trigger(input: &str, trigger_pos: usize) -> &str {
    input.get(trigger_pos + 1..).unwrap_or("")
}

fn filter_candidates(items: &[String], query: &str, max: usize) -> Vec<String> {
    let q = query.trim();
    let mut out: Vec<String> = items
        .iter()
        .filter(|s| q.is_empty() || s.starts_with(q))
        .take(max)
        .cloned()
        .collect();
    if out.is_empty() && !q.is_empty() {
        // Substring fallback (still bounded): first `max` matches.
        out = items
            .iter()
            .filter(|s| s.contains(q))
            .take(max)
            .cloned()
            .collect();
    }
    out
}

/// Walk `root` respecting .gitignore; relative POSIX paths.
pub fn collect_repo_paths(root: &Path, max: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut wb = WalkBuilder::new(root);
    wb.standard_filters(true);
    wb.hidden(false);
    for ent in wb.build().flatten() {
        let path = ent.path();
        if path == root {
            continue;
        }
        let Ok(rel) = path.strip_prefix(root) else {
            continue;
        };
        let s = rel.to_string_lossy().replace('\\', "/");
        if s.is_empty() {
            continue;
        }
        out.push(s);
        if out.len() >= max {
            break;
        }
    }
    out.sort();
    out.dedup();
    out
}

/// REPL slash commands + common `cantrik` CLI hints for `#` palette.
pub fn hash_palette_items() -> Vec<String> {
    [
        "/help",
        "/cost",
        "/memory",
        "/doctor",
        "/health",
        "/plan",
        "/agents",
        "/visualize",
        "/visualize architecture",
        "/exit",
        "cantrik ask …",
        "cantrik plan …",
        "cantrik index",
        "cantrik configure",
        "cantrik doctor",
        "cantrik health",
        "cantrik search …",
        "cantrik session list",
        "cantrik file read …",
        "cantrik exec …",
        "cantrik diff",
        "cantrik review",
        "cantrik workspace branch start …",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

pub fn new_path_completion(
    trigger_pos: usize,
    cwd: &Path,
    cache: &mut Vec<String>,
) -> ReplCompletion {
    if cache.is_empty() {
        *cache = collect_repo_paths(cwd, MAX_PATH_SCAN);
    }
    ReplCompletion {
        kind: CompletionKind::Path,
        trigger_pos,
        all_items: cache.clone(),
        filtered: Vec::new(),
        selected: 0,
    }
}

pub fn new_hash_completion(trigger_pos: usize) -> ReplCompletion {
    ReplCompletion {
        kind: CompletionKind::Hash,
        trigger_pos,
        all_items: hash_palette_items(),
        filtered: Vec::new(),
        selected: 0,
    }
}

/// If input still has `@` / `#` at `trigger_pos`, refresh; else return `None`.
pub fn sync_completion(input: &str, mut cur: ReplCompletion) -> Option<ReplCompletion> {
    let ch = input.as_bytes().get(cur.trigger_pos).copied()?;
    let expected = match cur.kind {
        CompletionKind::Path => b'@',
        CompletionKind::Hash => b'#',
    };
    if ch != expected {
        return None;
    }
    cur.refresh_from_input(input);
    Some(cur)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_prefix() {
        let v = vec!["a".into(), "ab".into(), "bc".into()];
        let f = filter_candidates(&v, "a", 10);
        assert_eq!(f, vec!["a", "ab"]);
    }
}
