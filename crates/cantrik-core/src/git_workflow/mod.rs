//! Git-native workflow helpers (Sprint 16): branches, commit messages, PR via `gh`, pre-commit review prompts.
//!
//! Writable git operations run only from dedicated CLI subcommands with `--approve`, not via `tool_git` allowlist.

mod branch;
mod commit;
mod pr;
mod review;
mod run;

pub use branch::{create_feature_branch, is_worktree_dirty, sanitize_task_slug};
pub use commit::{diff_staged, diff_worktree, git_commit_with_message, propose_commit_message};
pub use pr::{
    gh_available, looks_like_github_origin, origin_url, pr_create_dry_run_hint, run_gh_pr_create,
};
pub use review::build_review_prompt;
pub use run::{GitWorkflowError, git_write};
