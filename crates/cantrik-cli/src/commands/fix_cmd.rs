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

    if (run_agents || run_experiment) && (!approve || !fetch) {
        eprintln!("fix: --run-agents / --run-experiment require --approve and --fetch");
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
        println!("     (or `cantrik fix … --approve --fetch --run-agents`; timeout: CANTRIK_FIX_AGENT_TIMEOUT_SEC).");
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
        eprintln!("fix: pass --fetch together with --approve to execute step 1 (fetch issue HTML).");
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
