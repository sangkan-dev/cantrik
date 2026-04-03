use clap::Parser;
use std::process::ExitCode;

use cantrik_core::config::{load_merged_config, resolve_config_paths};

#[derive(Debug, Parser)]
#[command(name = "cantrik", version, about = "Cantrik CLI")]
struct Cli {
    #[arg(long, help = "Print config resolution paths")]
    debug_config: bool,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let cwd = match std::env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("failed to determine current directory: {error}");
            return ExitCode::FAILURE;
        }
    };

    if cli.debug_config {
        let paths = resolve_config_paths(&cwd);
        println!("global config : {}", paths.global.display());
        println!("project config: {}", paths.project.display());
    }

    if let Err(error) = load_merged_config(&cwd) {
        eprintln!("failed to load config: {error}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
