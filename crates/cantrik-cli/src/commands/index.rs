use std::path::Path;
use std::process::ExitCode;

use cantrik_core::config::AppConfig;

pub(crate) fn run(_config: &AppConfig, path: Option<&Path>) -> ExitCode {
    let display = path
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| ".".to_string());
    println!("index: (scaffold) indexer not wired — path: {display}");
    ExitCode::SUCCESS
}
