//! Wrap `run_command` in bubblewrap (Linux) or pass-through with macOS notes.
//!
//! Enterprise backlog: stronger isolation (gVisor runsc, Firecracker) should stay behind admin-only
//! config and feature flags; bubblewrap remains the default portable path.

use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::Command;

use crate::config::SandboxLevel;

/// When `sandbox.level = "container"` and this env names a `runsc` binary, Cantrik wraps exec with
/// `runsc run -- <program> …` (admin must ensure runsc/rootless policy fits the host).
pub const ENV_RUNSC_BIN: &str = "CANTRIK_RUNSC_BIN";

/// `CANTRIK_SANDBOX=0` disables sandbox wrapping (developer escape hatch; insecure).
pub const ENV_DISABLE_SANDBOX: &str = "CANTRIK_SANDBOX";

#[cfg(target_os = "linux")]
const BWRAP_BIN: &str = "bwrap";

fn sandbox_disabled_by_env() -> bool {
    matches!(
        std::env::var_os(ENV_DISABLE_SANDBOX).as_deref(),
        Some(s) if s == "0"
    )
}

/// Build a [`Command`] for `program` + ` argv` under `cwd` and sandbox `level`.
pub fn command_for_exec(
    program: &str,
    argv: &[String],
    cwd: &Path,
    level: SandboxLevel,
) -> Result<Command, String> {
    let cwd = cwd
        .canonicalize()
        .map_err(|e| format!("exec: canonicalize cwd {}: {e}", cwd.display()))?;
    let cwd_os: OsString = cwd.as_os_str().into();

    match level {
        SandboxLevel::Container => {
            let runsc = std::env::var_os(ENV_RUNSC_BIN).filter(|s| !s.is_empty());
            let Some(bin) = runsc else {
                return Err(format!(
                    "sandbox level 'container' requires {ENV_RUNSC_BIN} pointing to a runsc binary (gVisor); see CONTRIBUTING (enterprise sandbox). Otherwise use 'restricted' or 'none'"
                ));
            };
            runsc_command(&bin, program, argv, &cwd)
        }
        SandboxLevel::None => Ok(direct_command(program, argv, &cwd)),
        SandboxLevel::Restricted => {
            if sandbox_disabled_by_env() {
                return Ok(direct_command(program, argv, &cwd));
            }
            restricted_command(program, argv, &cwd_os)
        }
    }
}

fn direct_command(program: &str, argv: &[String], cwd: &Path) -> Command {
    let mut c = Command::new(program);
    c.args(argv);
    c.current_dir(cwd);
    c
}

/// Minimal `runsc run -- program argv…` wrapper; namespace/bind mounts are operator responsibility.
fn runsc_command(
    runsc_bin: &OsStr,
    program: &str,
    argv: &[String],
    cwd: &Path,
) -> Result<Command, String> {
    let mut c = Command::new(runsc_bin);
    c.arg("run");
    if let Ok(extra) = std::env::var("CANTRIK_RUNSC_RUN_ARGS") {
        for w in extra.split_whitespace() {
            c.arg(w);
        }
    }
    c.arg("--");
    c.arg(program);
    for a in argv {
        c.arg(a);
    }
    c.current_dir(cwd);
    Ok(c)
}

#[cfg(target_os = "linux")]
fn restricted_command(program: &str, argv: &[String], cwd: &OsString) -> Result<Command, String> {
    if which_binary(BWRAP_BIN).is_none() {
        return Err(
            "restricted sandbox requires 'bwrap' (bubblewrap) on PATH; install bubblewrap or set sandbox.level=\"none\" or CANTRIK_SANDBOX=0 (insecure)".into(),
        );
    }

    let mut cmd = Command::new(BWRAP_BIN);
    // Read-only root with writable cwd overlay; no network namespaces where possible.
    cmd.args([
        "--ro-bind",
        "/",
        "/",
        "--tmpfs",
        "/tmp",
        "--proc",
        "/proc",
        "--dev",
        "/dev",
        "--bind",
    ]);
    cmd.arg(cwd);
    cmd.arg(cwd);
    cmd.args(["--chdir"]);
    cmd.arg(cwd);
    cmd.args([
        "--unshare-pid",
        "--die-with-parent",
        "--unshare-net",
        "--new-session",
        "--",
        program,
    ]);
    for a in argv {
        cmd.arg(a);
    }
    Ok(cmd)
}

#[cfg(not(target_os = "linux"))]
fn restricted_command(program: &str, argv: &[String], cwd: &OsString) -> Result<Command, String> {
    if sandbox_disabled_by_env() {
        let p = Path::new(cwd);
        return Ok(direct_command(program, argv, p));
    }
    Err(
        "restricted sandbox on this platform needs CANTRIK_SANDBOX=0 to run without bubblewrap (macOS: use with care; Linux bubblewrap is supported)".into(),
    )
}

#[cfg(target_os = "linux")]
fn which_binary(name: &str) -> Option<std::path::PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            let p = dir.join(name);
            if p.is_file() { Some(p) } else { None }
        })
    })
}
