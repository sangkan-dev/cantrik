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
    /// Structured plan (JSON) under `.cantrik/plan-state.json`; `--run` runs evaluate/re-plan loop.
    Plan {
        #[arg(long)]
        status: bool,
        #[arg(long)]
        run: bool,
        #[arg(
            trailing_var_arg = true,
            required_unless_present = "status",
            value_name = "TASK"
        )]
        task: Vec<String>,
    },
    /// LLM proposes writes, run `[planning]` test command, revert checkpoints on failure.
    Experiment {
        #[arg(long)]
        approve: bool,
        #[arg(required = true, trailing_var_arg = true, value_name = "GOAL")]
        goal: Vec<String>,
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
    /// SQLite session memory: list sessions or show recent transcript for this project.
    Session {
        #[command(subcommand)]
        sub: SessionCommand,
    },
    /// Read/write files (writes print a diff; use `--approve` to apply).
    File {
        #[command(subcommand)]
        sub: FileCommand,
    },
    /// Run a command under `[sandbox]` policy (use `--approve` after reviewing the dry-run).
    Exec {
        #[arg(long)]
        approve: bool,
        #[arg(required = true, trailing_var_arg = true, value_name = "COMMAND")]
        argv: Vec<String>,
    },
    /// Run ripgrep (`rg`) for text search (distinct from vector `search`).
    Rgrep {
        #[arg(required = true, trailing_var_arg = true, value_name = "RG_ARGS")]
        args: Vec<String>,
    },
    /// Read-only git (allowlisted subcommands only).
    Git {
        #[arg(required = true, trailing_var_arg = true, value_name = "GIT_ARGS")]
        args: Vec<String>,
    },
    /// HTTP GET (`--approve` required).
    Fetch {
        url: String,
        #[arg(long)]
        approve: bool,
        #[arg(long, default_value_t = 2_000_000_u64)]
        max_bytes: u64,
    },
    /// Restore files from a pre-write checkpoint under `.cantrik/checkpoints/`.
    Rollback {
        /// List checkpoints for this project.
        #[arg(long)]
        list: bool,
        /// Checkpoint id (`001`) or folder substring; omit to restore the latest.
        #[arg(value_name = "ID")]
        id: Option<String>,
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

#[derive(Debug, Subcommand)]
pub enum SessionCommand {
    /// List sessions stored for the current project fingerprint.
    List,
    /// Show the active session transcript (latest session for cwd).
    Show {
        /// Max recent messages to print.
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
}

#[derive(Debug, Subcommand)]
pub enum FileCommand {
    /// Print file contents (capped by `[memory].max_file_read_bytes`).
    Read {
        #[arg(value_name = "PATH")]
        path: PathBuf,
    },
    /// Print unified diff vs stdin (or `--content-file`); pass `--approve` to write.
    Write {
        #[arg(value_name = "PATH")]
        path: PathBuf,
        #[arg(long)]
        content_file: Option<PathBuf>,
        #[arg(long)]
        approve: bool,
    },
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
