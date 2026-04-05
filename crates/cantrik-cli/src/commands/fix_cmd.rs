//! `cantrik fix` — guided workflow from an issue URL (Phase 5 foundation; not a full autonomous SWE loop).

use std::path::Path;
use std::process::ExitCode;

use cantrik_core::config::AppConfig;
use cantrik_core::tool_system::DEFAULT_MAX_FETCH_BYTES;

use super::agents_cmd;
use super::fetch_cmd;

fn is_github_issue_url(u: &str) -> bool {
    let lower = u.to_ascii_lowercase();
    (lower.contains("github.com") || lower.contains("github.io"))
        && (lower.contains("/issues/") || lower.contains("#issue-"))
}

/// MVP: print a concrete recipe; optionally `--approve --fetch` to pull the issue HTML via `fetch`;
/// optional `--run-agents` runs the multi-agent orchestrator after a successful fetch.
pub async fn run(
    cwd: &Path,
    config: &AppConfig,
    issue_url: &str,
    fetch: bool,
    approve: bool,
    run_agents: bool,
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

    if run_agents && (!approve || !fetch) {
        eprintln!("fix: --run-agents requires --approve and --fetch");
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
        println!("     (or use `cantrik fix … --approve --fetch --run-agents` to chain after fetch.)");
    }
    println!(
        "  3) Ship:      cantrik experiment --approve \"…\"   # or manual commit + cantrik pr create --approve"
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
    if !run_agents {
        return ExitCode::SUCCESS;
    }

    println!();
    println!("--- fix: running agents (orchestrator) ---");
    println!();
    let goal = format!(
        "Address issue context for {u}: from the fetched issue page (stdout above), summarize goal, constraints, files to touch, and concrete next steps for this repository."
    );
    agents_cmd::run(config, cwd, &goal, false, None, false).await
}
