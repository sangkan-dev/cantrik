use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

use cantrik_core::config::AppConfig;
use cantrik_core::tool_system::tool_git;

pub(crate) fn run(config: &AppConfig, cwd: &Path, args: Vec<String>) -> ExitCode {
    match tool_git(config, &args, cwd) {
        Ok(out) => {
            let _ = std::io::stdout().write_all(&out.stdout);
            let _ = std::io::stderr().write_all(&out.stderr);
            ExitCode::from(out.status.code().map(|c| c as u8).unwrap_or(1))
        }
        Err(e) => {
            eprintln!("git: {e}");
            ExitCode::FAILURE
        }
    }
}
