//! `cantrik fix` — guided workflow from an issue URL (Phase 5 foundation; not a full autonomous SWE loop).

use std::path::Path;
use std::process::ExitCode;
use std::time::Duration;

use cantrik_core::config::AppConfig;
use cantrik_core::tool_system::DEFAULT_MAX_FETCH_BYTES;
use tokio::time::timeout;

use super::agents_cmd;
use super::experiment_cmd;
use super::fetch_cmd;

fn is_github_issue_url(u: &str) -> bool {
    let lower = u.to_ascii_lowercase();
    (lower.contains("github.com") || lower.contains("github.io"))
        && (lower.contains("/issues/") || lower.contains("#issue-"))
}

fn fix_agent_timeout_sec() -> u64 {
    std::env::var("CANTRIK_FIX_AGENT_TIMEOUT_SEC")
        .ok()
        .and_then(|s| s.parse().ok())
        .filter(|&n| n > 0)
        .unwrap_or(900)
}

pub(crate) fn validate_fix_flags(
    run_agents: bool,
    run_experiment: bool,
    approve: bool,
    fetch: bool,
) -> Result<(), &'static str> {
    if (run_agents || run_experiment) && (!approve || !fetch) {
        return Err("fix: --run-agents / --run-experiment require --approve and --fetch");
    }
    Ok(())
}

/// MVP: print a concrete recipe; `--approve --fetch` runs fetch; optional `--run-agents` and `--run-experiment` chain after success.
pub async fn run(
    cwd: &Path,
    config: &AppConfig,
    issue_url: &str,
    fetch: bool,
    approve: bool,
    run_agents: bool,
    run_experiment: bool,
) -> ExitCode {
    let u = issue_url.trim();
    if u.is_empty() {
        eprintln!("fix: issue URL required");
        return ExitCode::from(2);
    }
    if !(u.starts_with("http://") || u.starts_with("https://")) {
        eprintln!("fix: expected http(s) URL");
        return ExitCode::from(2);
    }

    if let Err(msg) = validate_fix_flags(run_agents, run_experiment, approve, fetch) {
        eprintln!("{msg}");
        return ExitCode::from(2);
    }

    let gh = is_github_issue_url(u);
    println!("Issue: {u}");
    if gh {
        println!("(detected GitHub-style issue URL — adapt commands if your host differs.)");
    }
    println!();
    println!("Guided workflow (MVP — you stay in control):");
    println!("  1) Context:   cantrik fetch {u} --approve");
    println!(
        "  2) Implement: cantrik agents \"Address issue: {u} — summarize goal and constraints from fetch output.\""
    );
    if run_agents {
        println!(
            "     (or `cantrik fix … --approve --fetch --run-agents`; timeout: CANTRIK_FIX_AGENT_TIMEOUT_SEC)."
        );
    }
    println!(
        "  3) Ship:      cantrik experiment --approve \"…\"   # or `cantrik fix … --run-experiment` after fetch"
    );
    println!();
    println!("Project root for session tools: {}", cwd.display());

    if !approve {
        println!();
        println!(
            "Dry-run only. Re-run with --approve --fetch to download the issue page (cap default fetch bytes)."
        );
        return ExitCode::SUCCESS;
    }

    if !fetch {
        eprintln!(
            "fix: pass --fetch together with --approve to execute step 1 (fetch issue HTML)."
        );
        return ExitCode::from(2);
    }

    let max_bytes = DEFAULT_MAX_FETCH_BYTES;
    let code = fetch_cmd::run(config, u, true, max_bytes).await;
    if code != ExitCode::SUCCESS {
        return code;
    }

    if run_agents {
        println!();
        println!("--- fix: running agents (orchestrator) ---");
        println!();
        let goal = format!(
            "Address issue context for {u}: from the fetched issue page (stdout above), summarize goal, constraints, files to touch, and concrete next steps for this repository."
        );
        let secs = fix_agent_timeout_sec();
        let fut = agents_cmd::run(config, cwd, &goal, false, None, false);
        let agent_code = match timeout(Duration::from_secs(secs), fut).await {
            Ok(c) => c,
            Err(_) => {
                eprintln!("fix: agents timed out after {secs}s (CANTRIK_FIX_AGENT_TIMEOUT_SEC)");
                ExitCode::from(124)
            }
        };
        if agent_code != ExitCode::SUCCESS {
            return agent_code;
        }
    }

    if run_experiment {
        println!();
        println!("--- fix: running experiment (writes + test; --approve) ---");
        println!();
        let exp_goal = format!(
            "Issue context: {u}. Propose minimal, safe code changes to address the issue; prefer small patches and existing project conventions. If unclear, return empty writes."
        );
        return experiment_cmd::run(config, cwd, &exp_goal, true).await;
    }

    ExitCode::SUCCESS
}

/// Local HTTP fixture for CI: `fix --approve --fetch` without LLM agents.
#[cfg(test)]
mod fetch_integration {
    use std::path::Path;
    use std::process::ExitCode;

    use cantrik_core::config::AppConfig;
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::run;

    #[tokio::test]
    async fn fix_approve_fetch_reaches_local_http() {
        let srv = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                "<html><head><title>Fixture issue</title></head><body>ok</body></html>",
            ))
            .mount(&srv)
            .await;

        let url = format!("{}/issues/1", srv.uri());
        let cfg = AppConfig::default();
        let tmp = std::env::temp_dir();
        let code = run(Path::new(&tmp), &cfg, &url, true, true, false, false).await;
        assert_eq!(
            code,
            ExitCode::SUCCESS,
            "fix --approve --fetch should succeed against local fixture HTTP"
        );
    }

    /// Regression: body matches checked-in fixture (`tests/fixtures/…`) — stable “pinned” content without live hosts.
    #[tokio::test]
    async fn fix_approve_fetch_reaches_fixture_file() {
        let body = include_str!("../../../../tests/fixtures/cantrik-fix-issue-sample.html");
        let srv = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body.to_string()))
            .mount(&srv)
            .await;

        let url = format!("{}/issues/99", srv.uri());
        let cfg = AppConfig::default();
        let tmp = std::env::temp_dir();
        let code = run(Path::new(&tmp), &cfg, &url, true, true, false, false).await;
        assert_eq!(
            code,
            ExitCode::SUCCESS,
            "fix --approve --fetch should succeed when mock serves pinned fixture HTML"
        );
    }

    /// When `CANTRIK_FIX_E2E_HTTP_URL` is set, assert `fix --approve --fetch` against that URL (opt-in; not default CI).
    #[tokio::test]
    async fn fix_optional_pinned_http_url_from_env() {
        let Ok(url) = std::env::var("CANTRIK_FIX_E2E_HTTP_URL") else {
            return;
        };
        let url = url.trim();
        if url.is_empty() {
            return;
        }
        let cfg = AppConfig::default();
        let tmp = std::env::temp_dir();
        let code = run(Path::new(&tmp), &cfg, url, true, true, false, false).await;
        assert_eq!(
            code,
            ExitCode::SUCCESS,
            "fix --approve --fetch against CANTRIK_FIX_E2E_HTTP_URL"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_github_issue_url() {
        assert!(is_github_issue_url(
            "https://github.com/sangkan-dev/cantrik/issues/1"
        ));
        assert!(!is_github_issue_url("https://example.com/ticket/1"));
    }

    #[test]
    fn validate_fix_flags_blocks_chained_without_approve_fetch() {
        assert!(validate_fix_flags(true, false, false, false).is_err());
        assert!(validate_fix_flags(false, true, true, false).is_err());
        assert!(validate_fix_flags(true, false, true, true).is_ok());
    }
}
