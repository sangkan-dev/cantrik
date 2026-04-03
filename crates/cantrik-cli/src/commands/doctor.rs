use std::path::Path;
use std::process::ExitCode;

use cantrik_core::config::{load_merged_config, resolve_config_paths};

pub(crate) fn run(cwd: &Path) -> ExitCode {
    let paths = resolve_config_paths(cwd);

    println!("doctor: Cantrik {}", env!("CARGO_PKG_VERSION"));
    println!("  global config : {}", paths.global.display());
    println!("  project config: {}", paths.project.display());

    match load_merged_config(cwd) {
        Ok(config) => {
            println!("  config load: OK");
            if let Some(lang) = config.ui.language.as_deref() {
                println!("  ui.language  : {lang}");
            }
            if let Some(p) = config.llm.provider.as_deref() {
                println!("  llm.provider : {p}");
            }
            if let Some(m) = config.llm.model.as_deref() {
                println!("  llm.model    : {m}");
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("  config load: FAILED — {error}");
            // Doctor reports failure but exits 1 so CI/scripts can detect.
            ExitCode::from(1)
        }
    }
}
