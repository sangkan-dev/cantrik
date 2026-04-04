use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

use cantrik_core::checkpoint::{read_checkpoint_seq, revert_checkpoints_after_seq};
use cantrik_core::config::{AppConfig, effective_experiment_test_command, effective_sandbox_level};
use cantrik_core::planning::parse_experiment_writes;
use cantrik_core::tool_system::{ExecApproval, tool_run_command, tool_write_file};
use cantrik_core::tools::WriteApproval;

use super::session_llm;

pub(crate) async fn run(config: &AppConfig, cwd: &Path, goal: &str, approve: bool) -> ExitCode {
    if goal.trim().is_empty() {
        eprintln!("experiment: empty goal");
        return ExitCode::FAILURE;
    }

    let argv = effective_experiment_test_command(&config.planning);
    if !approve {
        eprintln!("experiment (dry-run): goal={goal:?}");
        eprintln!(
            "experiment: would call LLM for JSON writes, then run {:?}",
            argv
        );
        eprintln!(
            "experiment: pass --approve to execute (writes + test + auto-revert on failure)."
        );
        return ExitCode::SUCCESS;
    }

    let start_seq = read_checkpoint_seq(cwd);

    let prompt = format!(
        "You are in EXPERIMENT mode for a local codebase. Reply with ONLY valid JSON (no fences):\n\
{{\"writes\":[{{\"path\":\"relative/path.ext\",\"content\":\"full file text\"}}],\"rationale\":\"short\"}}\n\
Use empty writes array if no file changes. Paths are relative to project root.\n\n\
Goal:\n{}",
        goal.trim()
    );

    let raw = match session_llm::complete_with_session(cwd, config, &prompt).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("experiment: LLM error: {e}");
            return ExitCode::FAILURE;
        }
    };

    let parsed = parse_experiment_writes(&raw);
    if parsed.writes.is_empty() {
        eprintln!("experiment: model returned no writes; skipping file changes.");
    }

    for w in &parsed.writes {
        let path = Path::new(&w.path);
        if let Err(e) = tool_write_file(
            config,
            cwd,
            path,
            &w.content,
            WriteApproval::user_confirmed_after_reviewing_diff(),
        ) {
            eprintln!("experiment: write {} failed: {e}", w.path);
            if let Err(r) = revert_checkpoints_after_seq(cwd, start_seq) {
                eprintln!("experiment: rollback failed: {r}");
            } else {
                eprintln!("experiment: reverted checkpoints after seq {start_seq}.");
            }
            return ExitCode::FAILURE;
        }
    }

    if parsed.writes.is_empty() {
        // Still run tests to validate baseline? PRD implies change then test — skip if no writes.
        eprintln!("experiment: no writes applied; not running test command.");
        return ExitCode::SUCCESS;
    }

    let program = argv[0].clone();
    let args: Vec<String> = argv[1..].to_vec();
    let level = effective_sandbox_level(&config.sandbox);
    eprintln!(
        "experiment: running test command (sandbox={level:?}): {program} {:?}",
        args
    );

    let test_out = match tool_run_command(
        config,
        &program,
        &args,
        cwd,
        ExecApproval::user_approved_exec(),
    ) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("experiment: test invocation failed: {e}");
            if let Err(r) = revert_checkpoints_after_seq(cwd, start_seq) {
                eprintln!("experiment: rollback failed: {r}");
            } else {
                eprintln!("experiment: reverted checkpoints after seq {start_seq}.");
            }
            return ExitCode::FAILURE;
        }
    };

    let _ = std::io::stdout().write_all(&test_out.stdout);
    let _ = std::io::stderr().write_all(&test_out.stderr);

    let ok = test_out.status.success();
    if !ok {
        eprintln!("experiment: test command failed; reverting...");
        if let Err(r) = revert_checkpoints_after_seq(cwd, start_seq) {
            eprintln!("experiment: rollback failed: {r}");
            return ExitCode::FAILURE;
        }
        eprintln!("experiment: reverted to state before seq {}.", start_seq);
        return ExitCode::from(test_out.status.code().map(|c| c as u8).unwrap_or(1));
    }

    println!("experiment: tests passed; changes kept.");
    ExitCode::SUCCESS
}
