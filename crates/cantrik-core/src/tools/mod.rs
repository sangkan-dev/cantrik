//! File helpers: capped read, unified diff, gated write (Sprint 7).

mod read;
mod write;

pub use read::{ToolError, read_file_capped};
pub use write::{WriteApproval, commit_write, diff_for_new_contents, unified_diff};
