//! Capability tokens only constructible after explicit user approval (CLI / future agent).

/// Proof the user approved executing a subprocess after reviewing the command.
pub struct ExecApproval(());

impl ExecApproval {
    /// Call only from CLI after `--approve` (or tests via hidden API).
    #[doc(hidden)]
    pub fn user_approved_exec() -> Self {
        Self(())
    }
}

/// Proof the user approved an outbound HTTP fetch after reviewing the URL.
pub struct NetworkApproval(());

impl NetworkApproval {
    /// Call only from CLI after `--approve` (or tests via hidden API).
    #[doc(hidden)]
    pub fn user_approved_network() -> Self {
        Self(())
    }
}
