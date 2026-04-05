//! `cantrik health` — local audit / clippy / test gate with timeouts (Sprint 19, PRD §4.14).

use std::path::Path;
use std::process::ExitCode;

use cantrik_core::config::{
    AppConfig, effective_audit_command, load_merged_config, resolve_config_paths,
};
use tokio::time::Duration;

#[derive(Debug, Clone)]
pub struct HealthCli {
    pub soft: bool,
    pub no_clippy: bool,
    pub no_test: bool,
    /// Per-check timeout (seconds).
    pub timeout_sec: u64,
}

pub struct HealthReport {
    pub lines: Vec<String>,
    pub any_fail: bool,
}

struct StepResult {
    name: &'static str,
    ok: bool,
    detail: String,
}

async fn run_argv(
    cwd: &Path,
    argv: &[String],
    timeout_secs: u64,
) -> Result<std::process::Output, String> {
    if argv.is_empty() {
        return Err("empty command".into());
    }
    let dur = Duration::from_secs(timeout_secs.max(1));
    let (prog, rest) = argv.split_first().expect("argv non-empty");
    let mut c = tokio::process::Command::new(prog);
    c.args(rest).current_dir(cwd).kill_on_drop(true);
    match tokio::time::timeout(dur, c.output()).await {
        Err(_) => Err(format!("timeout after {timeout_secs}s: {}", argv.join(" "))),
        Ok(Err(e)) => Err(format!("failed to run `{}`: {e}", argv.join(" "))),
        Ok(Ok(o)) => Ok(o),
    }
}

fn summarize_output(out: &std::process::Output, max_lines: usize) -> String {
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let mut buf = String::new();
    if !stdout.trim().is_empty() {
        let lines: Vec<&str> = stdout.lines().collect();
        let tail: Vec<_> = if lines.len() > max_lines {
            lines[lines.len().saturating_sub(max_lines)..].to_vec()
        } else {
            lines
        };
        buf.push_str(&tail.join("\n"));
    }
    if !stderr.trim().is_empty() {
        if !buf.is_empty() {
            buf.push_str("\n--- stderr ---\n");
        }
        let lines: Vec<&str> = stderr.lines().collect();
        let tail: Vec<_> = if lines.len() > max_lines {
            lines[lines.len().saturating_sub(max_lines)..].to_vec()
        } else {
            lines
        };
        buf.push_str(&tail.join("\n"));
    }
    buf.trim().to_string()
}

async fn run_step(
    cwd: &Path,
    name: &'static str,
    argv: &[String],
    timeout_secs: u64,
) -> StepResult {
    match run_argv(cwd, argv, timeout_secs).await {
        Err(e) => StepResult {
            name,
            ok: false,
            detail: e,
        },
        Ok(out) => {
            let ok = out.status.success();
            let mut detail = summarize_output(&out, 48);
            if detail.is_empty() {
                detail = if ok {
                    "exit 0".into()
                } else {
                    format!("exit {}", out.status)
                };
            } else if !ok {
                detail = format!("exit {}\n{detail}", out.status);
            }
            StepResult { name, ok, detail }
        }
    }
}

/// Build a text report (for `cantrik health` stdout or REPL `/health`).
pub async fn run_report(cwd: &Path, config: &AppConfig, cli: &HealthCli) -> HealthReport {
    let paths = resolve_config_paths(cwd);
    let mut lines = Vec::new();
    lines.push(format!(
        "health: Cantrik {} (project root: {})",
        env!("CARGO_PKG_VERSION"),
        cwd.display()
    ));
    lines.push(format!(
        "  config: {} + {}",
        paths.global.display(),
        paths.project.display()
    ));

    let mut steps: Vec<StepResult> = Vec::new();
    let t = cli.timeout_sec;

    let audit_argv = effective_audit_command(&config.intelligence);
    steps.push(run_step(cwd, "audit", &audit_argv, t).await);

    if !cli.no_clippy {
        let clippy = vec![
            "cargo".into(),
            "clippy".into(),
            "--workspace".into(),
            "--".into(),
            "-D".into(),
            "warnings".into(),
        ];
        steps.push(run_step(cwd, "clippy", &clippy, t).await);
    }

    if !cli.no_test {
        let test = vec![
            "cargo".into(),
            "test".into(),
            "--workspace".into(),
            "--lib".into(),
        ];
        steps.push(run_step(cwd, "test (workspace --lib)", &test, t).await);
    }

    let mut any_fail = false;
    for s in &steps {
        let status = if s.ok { "OK" } else { "FAIL" };
        lines.push(String::new());
        lines.push(format!("[{status}] {}", s.name));
        for line in s.detail.lines() {
            lines.push(format!("  {line}"));
        }
        if !s.ok {
            any_fail = true;
        }
    }

    lines.push(String::new());
    if any_fail {
        lines.push("health: summary: one or more checks failed.".into());
    } else {
        lines.push("health: summary: all checks passed.".into());
    }

    HealthReport { lines, any_fail }
}

pub async fn run(cwd: &Path, cli: &HealthCli) -> ExitCode {
    let config: AppConfig = match load_merged_config(cwd) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("health: failed to load config: {e}");
            return if cli.soft {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            };
        }
    };

    let report = run_report(cwd, &config, cli).await;
    for l in &report.lines {
        println!("{l}");
    }

    if report.any_fail {
        if cli.soft {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        }
    } else {
        ExitCode::SUCCESS
    }
}
