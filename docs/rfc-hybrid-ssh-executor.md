# RFC: Hybrid SSH / cloud executor (backlog)

## Status

**MVP (CLI):** `cantrik exec --remote` prints the `ssh` line from `[remote_exec]` (dry-run); with `--approve` it runs `ssh` locally. Full file sync, sandbox mapping on remote, and product “GA” for enterprise remain **behind review** and tests per this RFC.

## Motivation

Offload heavy tasks (large builds, GPU, private packages) to a **user-controlled** machine while keeping the Cantrik CLI as the orchestration front-end.

## Goals

- Opt-in only; default remains local execution.
- Explicit approval for any command or file sync that crosses the trust boundary.
- Clear audit trail (what ran, on which host, as which user).

## Non-goals

- Managed multi-tenant cloud for Cantrik.
- Implicit trust of remote environments without user configuration.

## Threat model (summary)

| Risk | Mitigation direction |
|------|----------------------|
| Hostile remote shell | Allowlist commands; no arbitrary `bash -c` without review step |
| Credential theft on remote | Never send cloud API keys to remote by default; document `SSH_AUTH_SOCK` / agent risks |
| MITM on SSH | Require `StrictHostKeyChecking`; document known_hosts workflow |
| Data exfiltration | Sync only explicit paths; size caps |

## Config sketch (future)

```toml
[remote_exec]
enabled = false
host = "build.example.com"
user = "builder"
# identity_file = "~/.ssh/id_ed25519"
```

CLI: `cantrik exec --remote …` (dry-run) and `cantrik exec --remote --approve …` (runs `ssh`). Timeout: `CANTRIK_REMOTE_EXEC_TIMEOUT_SEC` (default 3600).

## Open questions

- Transport: plain SSH vs `ssh` + `rsync` for workspace snapshot?
- How to map `[sandbox]` on remote (likely none or remote-side bubblewrap)?
- Session correlation: same `.cantrik/` fingerprint or separate “remote session” id?

## Decision

Implement only after maintainers accept this RFC and add integration tests for the minimal happy path (single `echo` over SSH with approval).
