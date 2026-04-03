//! Tool registry, permission tiers, sandboxed exec (Sprint 8).

mod approvals;
mod dispatch;
mod forbidden;
mod git_allow;
mod sandbox;
mod tier;

pub use approvals::{ExecApproval, NetworkApproval};
pub use dispatch::{
    DEFAULT_MAX_FETCH_BYTES, DEFAULT_MAX_TOOL_OUTPUT_BYTES, ToolSystemError, tool_git,
    tool_read_file, tool_run_command, tool_search_rg, tool_web_fetch, tool_write_file,
};
pub use dispatch::{resolve_path_in_project, resolve_write_target};
pub use forbidden::check_exec_argv;
pub use sandbox::{ENV_DISABLE_SANDBOX, command_for_exec};
pub use tier::{PermissionTier, ToolId, effective_tier};
