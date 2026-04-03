//! Command-line argument definitions (clap).

use std::ffi::OsString;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Global options available on every subcommand.
#[derive(Debug, Parser)]
pub struct GlobalOpts {
    /// Print where Cantrik resolves global and project config paths.
    #[arg(long, global = true)]
    pub debug_config: bool,
}

/// Root CLI parser for `cantrik`.
#[derive(Debug, Parser)]
#[command(
    name = "cantrik",
    version,
    about = "Cantrik — open-source AI CLI agent (Rust)"
)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalOpts,

    /// Watch the workspace and suggest changes when files change (not implemented yet).
    #[arg(long, hide = true)]
    pub watch: bool,

    /// Read the initial prompt from the system clipboard (not implemented yet).
    #[arg(long, hide = true)]
    pub from_clipboard: bool,

    /// Path to an image for vision models (not implemented yet).
    #[arg(long, hide = true)]
    pub image: Option<PathBuf>,

    #[command(subcommand)]
    pub cmd: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Ask a question (read-oriented; execution wiring comes in later sprints).
    Ask {
        /// Words of the question (clap captures remainder).
        #[arg(trailing_var_arg = true, required = true, value_name = "QUERY")]
        query: Vec<String>,
    },
    /// Plan a change before executing (planner wiring comes in later sprints).
    Plan {
        #[arg(trailing_var_arg = true, required = true, value_name = "TASK")]
        task: Vec<String>,
    },
    /// Index or refresh the codebase AST index (chunks + intra-file call graph under `.cantrik/index/ast/`).
    Index {
        /// Skip Ollama embedding + LanceDB step after AST index.
        #[arg(long)]
        no_vectors: bool,
        /// Project path to index (default: current directory).
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
    },
    /// Semantic search over the local vector index (requires Ollama + prior `cantrik index`).
    Search {
        /// Project root (default: current directory).
        #[arg(short = 'C', long = "project", value_name = "DIR")]
        project: Option<PathBuf>,
        /// Maximum number of results.
        #[arg(long, default_value_t = 10)]
        limit: usize,
        /// Query words (natural language).
        #[arg(required = true, trailing_var_arg = true, value_name = "QUERY")]
        query: Vec<String>,
    },
    /// Check Cantrik installation, config, and connectivity (expanded over sprints).
    Doctor,
    /// Print shell completions to stdout (write to a file or source from your shell).
    Completions {
        /// Target shell.
        #[arg(value_enum)]
        shell: CompletionShell,
    },
    /// Anything that is not a known subcommand is treated as a one-shot `ask` prompt (PRD: `cantrik "..."`).
    #[command(external_subcommand)]
    External(Vec<OsString>),
}

/// Shells supported by `clap_complete` for static completion scripts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum CompletionShell {
    Bash,
    Elvish,
    Fish,
    PowerShell,
    Zsh,
}

impl From<CompletionShell> for clap_complete::Shell {
    fn from(value: CompletionShell) -> Self {
        match value {
            CompletionShell::Bash => clap_complete::Shell::Bash,
            CompletionShell::Elvish => clap_complete::Shell::Elvish,
            CompletionShell::Fish => clap_complete::Shell::Fish,
            CompletionShell::PowerShell => clap_complete::Shell::PowerShell,
            CompletionShell::Zsh => clap_complete::Shell::Zsh,
        }
    }
}
