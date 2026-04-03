//! Tool ids and effective permission tier from config + builtins (PRD §5).

use crate::config::AppConfig;

/// Stable string ids for `[guardrails]` lists and registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolId {
    ReadFile,
    WriteFile,
    RunCommand,
    Search,
    Git,
    WebFetch,
}

impl ToolId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadFile => "read_file",
            Self::WriteFile => "write_file",
            Self::RunCommand => "run_command",
            Self::Search => "search",
            Self::Git => "git",
            Self::WebFetch => "web_fetch",
        }
    }

    fn builtin_tier(self) -> PermissionTier {
        match self {
            Self::ReadFile | Self::Search | Self::Git => PermissionTier::AutoApprove,
            Self::WriteFile | Self::RunCommand | Self::WebFetch => PermissionTier::RequireApproval,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionTier {
    Forbidden,
    RequireApproval,
    AutoApprove,
}

fn list_has_id(list: &[String], id: &str) -> bool {
    list.iter().any(|s| s == id || s == "*")
}

/// Effective tier: `forbidden` → `require_approval` → `auto_approve` → builtin default.
pub fn effective_tier(config: &AppConfig, tool: ToolId) -> PermissionTier {
    let id = tool.as_str();
    let g = &config.guardrails;
    if list_has_id(&g.forbidden, id) {
        return PermissionTier::Forbidden;
    }
    if list_has_id(&g.require_approval, id) {
        return PermissionTier::RequireApproval;
    }
    if list_has_id(&g.auto_approve, id) {
        return PermissionTier::AutoApprove;
    }
    tool.builtin_tier()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    fn cfg() -> AppConfig {
        AppConfig::default()
    }

    #[test]
    fn forbidden_overrides_auto_list() {
        let mut c = cfg();
        c.guardrails.forbidden = vec!["read_file".into()];
        c.guardrails.auto_approve = vec!["read_file".into()];
        assert_eq!(
            effective_tier(&c, ToolId::ReadFile),
            PermissionTier::Forbidden
        );
    }

    #[test]
    fn star_forbids_all() {
        let mut c = cfg();
        c.guardrails.forbidden = vec!["*".into()];
        assert_eq!(
            effective_tier(&c, ToolId::Search),
            PermissionTier::Forbidden
        );
    }
}
