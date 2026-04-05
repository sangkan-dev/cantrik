use std::io::Write;
use std::path::Path;
use std::process::ExitCode;
use std::time::Duration;

use cantrik_core::config::{AppConfig, effective_sandbox_level, remote_ssh_destination};
use cantrik_core::tool_system::{ExecApproval, tool_run_command};
use tokio::time::timeout;

use super::approval_record;

fn sh_quote_arg(s: &str) -> String {
    if s.is_empty() {
        return "''".into();
    }
    if s.chars()
        .all(|c| c.is_ascii_alphanumeric() || "/._+-=@:,^%".contains(c))
    {
        return s.to_string();
    }
    format!("'{}'", s.replace('\'', "'\"'\"'"))
}

fn format_remote_exec_line(
    config: &AppConfig,
    dest: &str,
    program: &str,
    args: &[String],
) -> String {
    let mut out = vec![
        "ssh".to_string(),
        "-o".into(),
        "BatchMode=yes".into(),
        "-o".into(),
        "ConnectTimeout=15".into(),
    ];
    if let Some(ref id) = config.remote_exec.identity_file
        && !id.trim().is_empty()
    {
        out.push("-i".into());
        out.push(sh_quote_arg(id.trim()));
    }
    for x in &config.remote_exec.extra_ssh_args {
        out.push(sh_quote_arg(x));
    }
    out.push(sh_quote_arg(dest));
    out.push("--".into());
    out.push(sh_quote_arg(program));
    for a in args {
        out.push(sh_quote_arg(a));
    }
    out.join(" ")
}

pub(crate) async fn run(
    config: &AppConfig,
    cwd: &Path,
    approve: bool,
    remote: bool,
    argv: Vec<String>,
) -> ExitCode {
    if argv.is_empty() {
        eprintln!("exec: missing command (example: cantrik exec --approve -- echo hello)");
        return ExitCode::FAILURE;
    }
    let program = argv[0].clone();
    let args: Vec<String> = argv[1..].to_vec();

    if remote {
        let Some(dest) = remote_ssh_destination(&config.remote_exec) else {
            eprintln!(
                "exec --remote: set [remote_exec].host in cantrik.toml (see docs/rfc-hybrid-ssh-executor.md)."
            );
            return ExitCode::FAILURE;
        };
        let preview = format_remote_exec_line(config, &dest, &program, &args);
        if !approve {
            eprintln!("exec (remote dry-run): would run:\n  {preview}");
            eprintln!("exec: pass --approve to run this ssh command.");
            return ExitCode::SUCCESS;
        }

        let mut cmd = tokio::process::Command::new("ssh");
        cmd.arg("-o")
            .arg("BatchMode=yes")
            .arg("-o")
            .arg("ConnectTimeout=15");
        if let Some(ref id) = config.remote_exec.identity_file
            && !id.trim().is_empty()
        {
            cmd.arg("-i").arg(id.trim());
        }
        for x in &config.remote_exec.extra_ssh_args {
            cmd.arg(x);
        }
        cmd.arg(&dest);
        cmd.arg("--");
        cmd.arg(&program);
        cmd.args(&args);
        cmd.current_dir(cwd);
        cmd.kill_on_drop(true);

        let timeout_secs: u64 = std::env::var("CANTRIK_REMOTE_EXEC_TIMEOUT_SEC")
            .ok()
            .and_then(|s| s.parse().ok())
            .filter(|&n| n > 0)
            .unwrap_or(3600);

        match timeout(Duration::from_secs(timeout_secs), cmd.output()).await {
            Err(_) => {
                eprintln!(
                    "exec: remote: timeout after {timeout_secs}s (CANTRIK_REMOTE_EXEC_TIMEOUT_SEC)"
                );
                ExitCode::from(124)
            }
            Ok(Err(e)) => {
                eprintln!("exec: remote: {e}");
                ExitCode::FAILURE
            }
            Ok(Ok(out)) => {
                let _ = std::io::stdout().write_all(&out.stdout);
                let _ = std::io::stderr().write_all(&out.stderr);
                ExitCode::from(out.status.code().map(|c| c as u8).unwrap_or(1))
            }
        }
    } else {
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
}
