//! Registry dispatch: tier checks, approvals, and tool implementations.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use reqwest::Client;
use thiserror::Error;

use crate::audit;
use crate::checkpoint;
use crate::config::{AppConfig, effective_sandbox_level, provenance_file_enabled};
use crate::plugins::{lua_runtime, wasm_runtime};
use crate::provenance;
use crate::tools::{ToolError as FileToolError, read_file_capped};
use crate::tools::{WriteApproval, commit_write};
use url::Url;

use super::approvals::{ExecApproval, NetworkApproval};
use super::forbidden::check_exec_argv;
use super::git_allow::check_git_args;
use super::sandbox::command_for_exec;
use super::tier::{PermissionTier, ToolId, effective_tier};

/// Cap captured stdout from ripgrep / subprocess helpers.
pub const DEFAULT_MAX_TOOL_OUTPUT_BYTES: usize = 2_000_000;

/// Max HTTP response body for `web_fetch`.
pub const DEFAULT_MAX_FETCH_BYTES: u64 = 2_000_000;

#[derive(Debug, Error)]
pub enum ToolSystemError {
    #[error("tool {0:?} is forbidden by guardrails")]
    Forbidden(ToolId),
    #[error("{0}")]
    Policy(String),
    #[error("path escapes project root: {0}")]
    PathOutsideProject(PathBuf),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("file tool: {0}")]
    File(#[from] FileToolError),
    #[error("http: {0}")]
    Http(String),
}

fn require_not_forbidden(config: &AppConfig, tool: ToolId) -> Result<(), ToolSystemError> {
    if effective_tier(config, tool) == PermissionTier::Forbidden {
        return Err(ToolSystemError::Forbidden(tool));
    }
    Ok(())
}

/// Ensure `path` resolves under `project_root` (after canonicalize).
pub fn resolve_path_in_project(
    path: &Path,
    project_root: &Path,
) -> Result<PathBuf, ToolSystemError> {
    let root = project_root
        .canonicalize()
        .map_err(|e| ToolSystemError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, e)))?;
    let p = path
        .canonicalize()
        .map_err(|e| ToolSystemError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, e)))?;
    if !p.starts_with(&root) {
        return Err(ToolSystemError::PathOutsideProject(p));
    }
    Ok(p)
}

/// Target path for a write (file may not exist yet): parent directory must lie under `project_root`.
pub fn resolve_write_target(path: &Path, project_root: &Path) -> Result<PathBuf, ToolSystemError> {
    let root = project_root.canonicalize()?;
    let target = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    let parent = target
        .parent()
        .ok_or_else(|| ToolSystemError::Policy("write path has no parent".into()))?;
    let parent_canon = parent.canonicalize()?;
    if !parent_canon.starts_with(&root) {
        return Err(ToolSystemError::PathOutsideProject(parent_canon));
    }
    Ok(target)
}

/// Read a file within `project_root` with tier + path checks.
pub fn tool_read_file(
    config: &AppConfig,
    project_root: &Path,
    path: &Path,
    max_bytes: u64,
) -> Result<String, ToolSystemError> {
    require_not_forbidden(config, ToolId::ReadFile)?;
    let p = resolve_path_in_project(path, project_root)?;
    Ok(read_file_capped(&p, max_bytes)?)
}

/// Write file bytes after tier check; `approval` is still mandatory (Sprint 7).
pub fn tool_write_file(
    config: &AppConfig,
    project_root: &Path,
    path: &Path,
    contents: &str,
    approval: WriteApproval,
) -> Result<(), ToolSystemError> {
    require_not_forbidden(config, ToolId::WriteFile)?;
    let p = resolve_write_target(path, project_root)?;
    let task = std::env::var("CANTRIK_TASK").ok();
    checkpoint::snapshot_before_write(project_root, &p, task.as_deref())
        .map_err(|e| ToolSystemError::Policy(format!("checkpoint: {e}")))?;
    commit_write(&p, contents, approval)?;
    let root = project_root.canonicalize().map_err(ToolSystemError::Io)?;
    let rel = p
        .strip_prefix(&root)
        .map(|x| x.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| p.to_string_lossy().into_owned());
    let line = audit::format_write_audit(
        &rel,
        config.llm.provider.as_deref(),
        config.llm.model.as_deref(),
        contents,
    );
    let _ = audit::append_audit_line(line.trim_end());
    if provenance_file_enabled(&config.audit) {
        let _ = provenance::append_provenance_record(project_root, &rel, config.llm.model.clone());
    }
    for msg in lua_runtime::after_write_messages(project_root, &rel) {
        eprintln!("plugin suggest: {msg}");
    }
    wasm_runtime::run_wasm_after_write_hooks(project_root);
    Ok(())
}

/// Run subprocess with `ExecApproval` and sandbox level from config.
pub fn tool_run_command(
    config: &AppConfig,
    program: &str,
    args: &[String],
    cwd: &Path,
    _approval: ExecApproval,
) -> Result<std::process::Output, ToolSystemError> {
    require_not_forbidden(config, ToolId::RunCommand)?;
    let mut argv = vec![program.to_string()];
    argv.extend_from_slice(args);
    check_exec_argv(&argv).map_err(|e: &str| ToolSystemError::Policy(e.into()))?;

    let level = effective_sandbox_level(&config.sandbox);
    let mut cmd = command_for_exec(program, args, cwd, level).map_err(ToolSystemError::Policy)?;
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    let out = cmd.output().map_err(ToolSystemError::Io)?;
    let line = audit::format_exec_audit(program, args);
    let _ = audit::append_audit_line(line.trim_end());
    Ok(out)
}

const RG_BIN: &str = "rg";

/// Run ripgrep; forwards `args` after binary name. stdout truncated to `max_out` bytes.
pub fn tool_search_rg(
    config: &AppConfig,
    args: &[String],
    cwd: &Path,
    max_out: usize,
) -> Result<std::process::Output, ToolSystemError> {
    require_not_forbidden(config, ToolId::Search)?;
    let mut cmd = Command::new(RG_BIN);
    cmd.args(args);
    cmd.current_dir(cwd);
    let out = cmd.output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ToolSystemError::Policy(
                "ripgrep executable 'rg' not found on PATH; install ripgrep".into(),
            )
        } else {
            ToolSystemError::Io(e)
        }
    })?;
    if out.stdout.len() > max_out {
        return Err(ToolSystemError::Policy(format!(
            "rg stdout exceeded {max_out} bytes"
        )));
    }
    Ok(out)
}

/// Full `git` argv after the `git` itself (e.g. `["log", "-1"]`).
pub fn tool_git(
    config: &AppConfig,
    git_args: &[String],
    cwd: &Path,
) -> Result<std::process::Output, ToolSystemError> {
    require_not_forbidden(config, ToolId::Git)?;
    check_git_args(git_args).map_err(|e: &str| ToolSystemError::Policy(e.into()))?;
    let mut cmd = Command::new("git");
    cmd.args(git_args);
    cmd.current_dir(cwd);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    cmd.output().map_err(ToolSystemError::Io)
}

async fn http_get_capped(url: &str, max_bytes: u64) -> Result<Vec<u8>, ToolSystemError> {
    let client = Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| ToolSystemError::Http(e.to_string()))?;
    let res = client
        .get(url)
        .header(
            "User-Agent",
            "cantrik/0.1 (https://github.com/sangkan/cantrik; web research MVP)",
        )
        .send()
        .await
        .map_err(|e| ToolSystemError::Http(e.to_string()))?;
    if !res.status().is_success() {
        return Err(ToolSystemError::Http(format!("HTTP {}", res.status())));
    }
    if let Some(n) = res.content_length()
        && n > max_bytes
    {
        return Err(ToolSystemError::Http(format!(
            "Content-Length {n} exceeds cap {max_bytes}"
        )));
    }
    let body = res
        .bytes()
        .await
        .map_err(|e| ToolSystemError::Http(e.to_string()))?;
    if body.len() as u64 > max_bytes {
        return Err(ToolSystemError::Http(format!(
            "response body exceeds {max_bytes} bytes"
        )));
    }
    Ok(body.to_vec())
}

/// HTTP GET with response size cap. Requires `NetworkApproval`.
pub async fn tool_web_fetch(
    config: &AppConfig,
    url: &str,
    max_bytes: u64,
    approval: NetworkApproval,
) -> Result<Vec<u8>, ToolSystemError> {
    require_not_forbidden(config, ToolId::WebFetch)?;
    let body = http_get_capped(url, max_bytes).await?;
    let line = audit::format_fetch_audit(url, body.len());
    let _ = audit::append_audit_line(line.trim_end());
    let _ = approval;
    Ok(body)
}

/// Same as [`tool_web_fetch`] but checked against `browse_page` guardrails.
pub async fn tool_browse_page(
    config: &AppConfig,
    url: &str,
    max_bytes: u64,
    approval: NetworkApproval,
) -> Result<Vec<u8>, ToolSystemError> {
    require_not_forbidden(config, ToolId::BrowsePage)?;
    let body = http_get_capped(url, max_bytes).await?;
    let line = audit::format_fetch_audit(url, body.len());
    let _ = audit::append_audit_line(line.trim_end());
    let _ = approval;
    Ok(body)
}

/// Same as [`tool_browse_page`] for documentation URLs (`fetch_docs` tool id).
pub async fn tool_fetch_docs(
    config: &AppConfig,
    url: &str,
    max_bytes: u64,
    approval: NetworkApproval,
) -> Result<Vec<u8>, ToolSystemError> {
    require_not_forbidden(config, ToolId::FetchDocs)?;
    let body = http_get_capped(url, max_bytes).await?;
    let line = audit::format_fetch_audit(url, body.len());
    let _ = audit::append_audit_line(line.trim_end());
    let _ = approval;
    Ok(body)
}

fn format_ddg_result_lines(html: &str, max_results: usize) -> String {
    let mut out = String::new();
    let mut count = 0usize;
    for part in html.split("result__a") {
        if count >= max_results {
            break;
        }
        let Some(idx) = part.find("href=\"") else {
            continue;
        };
        let href_part = &part[idx + 6..];
        let Some(end) = href_part.find('"') else {
            continue;
        };
        let link = &href_part[..end];
        if link.starts_with("http") {
            count += 1;
            out.push_str(&format!("{count}. {link}\n"));
        }
    }
    if out.is_empty() {
        "No parseable search results (page layout may have changed). Try `cantrik fetch <url> --approve` for a known documentation URL.\n".to_string()
    } else {
        out
    }
}

/// HTML search via Duck Duck Go (unofficial HTML endpoint; best-effort MVP).
pub async fn tool_web_search(
    config: &AppConfig,
    query: &str,
    max_results: usize,
    max_response_bytes: u64,
    approval: NetworkApproval,
) -> Result<String, ToolSystemError> {
    require_not_forbidden(config, ToolId::WebSearch)?;
    let _ = approval;
    let mut u = Url::parse("https://html.duckduckgo.com/html/")
        .map_err(|e| ToolSystemError::Http(e.to_string()))?;
    u.query_pairs_mut().append_pair("q", query);
    let body = http_get_capped(u.as_str(), max_response_bytes).await?;
    let html = String::from_utf8_lossy(&body);
    Ok(format_ddg_result_lines(&html, max_results.max(1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::tool_system::{ExecApproval, ToolId};

    #[test]
    fn ddg_format_extracts_links() {
        let html =
            r#"x result__a" href="http://example.com/a" y result__a" href="https://b.test/z""#;
        let s = super::format_ddg_result_lines(html, 5);
        assert!(s.contains("example.com"));
        assert!(s.contains("b.test"));
    }

    #[test]
    fn run_command_blocked_when_forbidden() {
        let mut c = AppConfig::default();
        c.guardrails
            .forbidden
            .push(ToolId::RunCommand.as_str().to_string());
        let cwd = std::env::current_dir().expect("cwd");
        let out = tool_run_command(
            &c,
            "true",
            &[],
            cwd.as_path(),
            ExecApproval::user_approved_exec(),
        );
        assert!(matches!(
            out,
            Err(ToolSystemError::Forbidden(ToolId::RunCommand))
        ));
    }

    #[cfg(unix)]
    #[test]
    fn path_outside_project_rejected_for_read() {
        let c = AppConfig::default();
        let tmp = std::env::temp_dir();
        let err = tool_read_file(&c, tmp.as_path(), Path::new("/etc/passwd"), 1024);
        assert!(matches!(err, Err(ToolSystemError::PathOutsideProject(_))));
    }

    #[test]
    fn write_twice_then_rollback_restores_first_content() {
        use crate::checkpoint::{apply_checkpoint, latest_checkpoint_dir};
        use crate::tools::WriteApproval;

        let root =
            std::env::temp_dir().join(format!("cantrik-dispatch-rollback-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src")).expect("mkdir");
        let root = root.canonicalize().expect("canon");

        let c = AppConfig::default();
        let rel = Path::new("src").join("roll.txt");
        let approve = WriteApproval::user_confirmed_after_reviewing_diff();

        tool_write_file(&c, &root, &rel, "first", approve).expect("write 1");
        tool_write_file(
            &c,
            &root,
            &rel,
            "second",
            WriteApproval::user_confirmed_after_reviewing_diff(),
        )
        .expect("write 2");

        let p = root.join(&rel);
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "second");

        let cp = latest_checkpoint_dir(&root)
            .expect("list")
            .expect("some checkpoint");
        apply_checkpoint(&root, &cp).expect("rollback");

        assert_eq!(std::fs::read_to_string(&p).unwrap(), "first");

        let _ = std::fs::remove_dir_all(&root);
    }
}
