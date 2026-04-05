use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

use cantrik_core::config::{AppConfig, effective_sandbox_level};
use cantrik_core::tool_system::{ExecApproval, tool_run_command};

use super::approval_record;

pub(crate) async fn run(
    config: &AppConfig,
    cwd: &Path,
    approve: bool,
    argv: Vec<String>,
) -> ExitCode {
    if argv.is_empty() {
        eprintln!("exec: missing command (example: cantrik exec --approve -- echo hello)");
        return ExitCode::FAILURE;
    }
    let program = argv[0].clone();
    let args: Vec<String> = argv[1..].to_vec();
    let level = effective_sandbox_level(&config.sandbox);

    if !approve {
        eprintln!("exec (dry-run): sandbox={level:?}, program={program}, args={args:?}");
        eprintln!("exec: pass --approve to run after reviewing the command.");
        return ExitCode::SUCCESS;
    }

    match tool_run_command(
        config,
        &program,
        &args,
        cwd,
        ExecApproval::user_approved_exec(),
    ) {
        Ok(out) => {
            let _ = std::io::stdout().write_all(&out.stdout);
            let _ = std::io::stderr().write_all(&out.stderr);
            let ok = out.status.success();
            let sum = format!("{program} {:?}", args);
            approval_record::record_if_adaptive(cwd, config, "run_command", ok, &sum).await;
            ExitCode::from(out.status.code().map(|c| c as u8).unwrap_or(1))
        }
        Err(e) => {
            eprintln!("exec: {e}");
            ExitCode::FAILURE
        }
    }
}
