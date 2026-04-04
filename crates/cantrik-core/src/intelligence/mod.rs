//! Intelligence tools: explain (blame + log), teach, dependency helpers (Sprint 17, PRD §4.20, §4.24–4.25).

mod deps;
mod explain;
mod teach;

pub use deps::{
    build_upgrade_suggestion_prompt, build_why_context, count_crate_mentions_in_manifests,
    parse_cargo_tree_invert_first_lines, read_cargo_lock_excerpt, run_cargo_audit,
    run_cargo_tree_depth, run_cargo_tree_invert,
};
pub use explain::{
    build_explain_why_prompt, collect_explain_context, extract_blame_commit_prefixes,
};
pub use teach::{
    apply_wiki_format, build_teach_prompt, gather_teach_context, gather_teach_context_from_parts,
};

use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IntelligenceError {
    #[error("git: {0}")]
    Git(String),
    #[error("cargo: {0}")]
    Cargo(String),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("{0}")]
    Msg(String),
}
