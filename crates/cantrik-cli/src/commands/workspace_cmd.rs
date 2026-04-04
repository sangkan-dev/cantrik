use std::path::Path;
use std::process::ExitCode;

use cantrik_core::config::{AppConfig, effective_git_branch_prefix, load_merged_config};
use cantrik_core::git_workflow::{
    create_feature_branch, diff_staged, git_commit_with_message, propose_commit_message,
};

#[derive(Debug, Clone, clap::Subcommand)]
pub enum WorkspaceBranchCommand {
    /// Create `feature/cantrik-<slug>` (or configured prefix).
    Start {
        #[arg(value_name = "SLUG")]
        slug: String,
        #[arg(long)]
        allow_dirty: bool,
    },
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum WorkspaceCommand {
    Branch {
        #[command(subcommand)]
        sub: WorkspaceBranchCommand,
    },
    /// Propose a commit message via LLM from staged diff; with `--approve`, run `git commit`.
    Commit {
        #[arg(long)]
        approve: bool,
        #[arg(long)]
        amend: bool,
        #[arg(long)]
        no_verify: bool,
        #[arg(long, value_name = "TEXT")]
        message: Option<String>,
    },
}

pub async fn run(cwd: &Path, sub: &WorkspaceCommand) -> ExitCode {
    let config = match load_merged_config(cwd) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("failed to load config: {e}");
            return ExitCode::FAILURE;
        }
    };
    match sub {
        WorkspaceCommand::Branch {
            sub: WorkspaceBranchCommand::Start { slug, allow_dirty },
        } => branch_start(cwd, &config, slug, *allow_dirty),
        WorkspaceCommand::Commit {
            approve,
            amend,
            no_verify,
            message,
        } => {
            commit_cmd(
                cwd,
                &config,
                *approve,
                *amend,
                *no_verify,
                message.as_deref(),
            )
            .await
        }
    }
}

fn branch_start(cwd: &Path, config: &AppConfig, slug: &str, allow_dirty: bool) -> ExitCode {
    let prefix = effective_git_branch_prefix(&config.git_workflow);
    match create_feature_branch(cwd, &prefix, slug, allow_dirty) {
        Ok(b) => {
            println!("{b}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("workspace branch: {e}");
            ExitCode::FAILURE
        }
    }
}

async fn commit_cmd(
    cwd: &Path,
    config: &AppConfig,
    approve: bool,
    amend: bool,
    no_verify: bool,
    message_override: Option<&str>,
) -> ExitCode {
    let diff = match diff_staged(cwd) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("workspace commit: {e}");
            return ExitCode::FAILURE;
        }
    };
    if diff.trim().is_empty() && !amend {
        eprintln!("workspace commit: no staged changes (`git add` first)");
        return ExitCode::from(2);
    }

    let msg = if let Some(m) = message_override.filter(|s| !s.is_empty()) {
        m.to_string()
    } else {
        match propose_commit_message(config, &diff).await {
            Ok(m) => m.trim().to_string(),
            Err(e) => {
                eprintln!("workspace commit: LLM: {e}");
                return ExitCode::FAILURE;
            }
        }
    };

    if !approve {
        println!("Proposed commit message:\n---\n{msg}\n---");
        println!("Run with --approve to execute `git commit`.");
        return ExitCode::SUCCESS;
    }

    match git_commit_with_message(cwd, &msg, amend, no_verify) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("workspace commit: {e}");
            ExitCode::FAILURE
        }
    }
}
