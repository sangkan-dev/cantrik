//! Semantic diff, git conflict hints, and collaboration helpers (Sprint 15).

mod diff_lines;
mod git_conflicts;
mod git_workspace;
mod index_map;
pub mod report;
mod risk;
mod tests_hint;

pub mod collab;

pub use collab::{
    CollabError, ContextBundleV1, SessionReplayFileV1, export_context_bundle,
    export_session_replay_json, format_replay_timeline, import_context_bundle,
    parse_session_replay, serialize_context_bundle, serialize_session_replay,
    write_handoff_markdown,
};
pub use diff_lines::changed_new_line_indices;
pub use git_conflicts::{ConflictEntry, list_conflicts};
pub use git_workspace::{GitWorkspaceError, changed_paths, show_blob, status_porcelain};
pub use index_map::{
    affected_chunks_for_path, callers_for_symbols, chunks_by_path, load_call_edges,
};
pub use report::{FileSemanticEntry, SemanticReport, build_semantic_report};
pub use risk::{RiskLevel, assess_risk};
pub use tests_hint::test_hints_for_changes;
