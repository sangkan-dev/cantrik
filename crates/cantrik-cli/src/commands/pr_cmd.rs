use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

use cantrik_core::config::{AppConfig, effective_pr_provider, load_merged_config};
use cantrik_core::git_workflow::{
    gh_available, looks_like_github_origin, origin_url, pr_create_dry_run_hint, run_gh_pr_create,
};

#[derive(Debug, Clone, clap::Subcommand)]
pub enum PrCommand {
    Create {
        #[arg(long)]
        approve: bool,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        body: Option<String>,
        #[arg(long)]
        body_file: Option<std::path::PathBuf>,
        #[arg(long)]
        draft: bool,
    },
}

pub fn run(cwd: &Path, config: &AppConfig, sub: &PrCommand) -> ExitCode {
    let PrCommand::Create {
        approve,
        title,
        body,
        body_file,
        draft,
    } = sub;
    pr_create(
        cwd,
        config,
        *approve,
        title.as_deref(),
        body.as_deref(),
        body_file.as_deref(),
        *draft,
    )
}

pub fn run_standalone(cwd: &Path, sub: &PrCommand) -> ExitCode {
    let config = match load_merged_config(cwd) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("failed to load config: {e}");
            return ExitCode::FAILURE;
        }
    };
    run(cwd, &config, sub)
}

fn pr_create(
    cwd: &Path,
    config: &AppConfig,
    approve: bool,
    title: Option<&str>,
    body: Option<&str>,
    body_file: Option<&Path>,
    draft: bool,
) -> ExitCode {
    let provider = effective_pr_provider(&config.git_workflow);
    if provider == "none" {
        eprintln!("pr create: pr_provider is none");
        return ExitCode::from(2);
    }
    let origin = match origin_url(cwd) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("pr create: {e}");
            return ExitCode::FAILURE;
        }
    };
    if !looks_like_github_origin(&origin) {
        eprintln!("pr create: origin not GitHub: {origin}");
        return ExitCode::from(2);
    }
    if !gh_available() {
        eprintln!("pr create: need gh CLI");
        return ExitCode::FAILURE;
    }
    if !approve {
        println!("{}", pr_create_dry_run_hint());
        if let Some(t) = title {
            println!("title: {t}");
        }
        return ExitCode::SUCCESS;
    }
    match run_gh_pr_create(cwd, title, body, body_file, draft) {
        Ok(out) => {
            let _ = std::io::stdout().write_all(&out.stdout);
            let _ = std::io::stderr().write_all(&out.stderr);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("pr create: {e}");
            ExitCode::FAILURE
        }
    }
}
