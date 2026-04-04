//! Command-line argument definitions (clap).

use std::ffi::OsString;
use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

pub use crate::commands::pr_cmd::PrCommand;
pub use crate::commands::web_cmd::WebCommand;
pub use crate::commands::workspace_cmd::WorkspaceCommand;

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
    /// Multi-agent orchestrator: Planner decompose, parallel ephemeral workers, synthesize, Builder stub.
    Agents {
        #[arg(long)]
        dry_run: bool,
        #[arg(long, value_name = "N")]
        max_parallel: Option<usize>,
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
    /// Queue a background goal for `cantrik daemon`, or `cantrik background resume <id>`.
    Background {
        /// Skip desktop/webhook/file notification when the job needs approval (default: notify).
        #[arg(long)]
        no_notify: bool,
        #[arg(required = true, trailing_var_arg = true, value_name = "ARGS")]
        args: Vec<String>,
    },
    /// List background jobs for this project (`--all` for every project in the DB).
    Status {
        #[arg(long)]
        all: bool,
        #[arg(long, default_value_t = 50_i64)]
        limit: i64,
    },
    /// Long-running worker: claims queued background jobs and runs a bounded LLM round each time.
    Daemon {
        #[arg(long, default_value_t = 2_u64)]
        poll_secs: u64,
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
    /// Install/list/update/remove skill packages from the local registry (Sprint 13).
    Skill {
        #[command(subcommand)]
        sub: SkillCommand,
    },
    /// Record and replay command sequences (Sprint 13, PRD §4.18).
    Macro {
        #[command(subcommand)]
        sub: MacroCommand,
    },
    /// Approximate LLM spend (UTC calendar month + active session for this cwd).
    Cost {
        /// Only print spend for the current SQLite session (latest for project fingerprint).
        #[arg(long)]
        session_only: bool,
    },
    /// Local server modes (MCP stdio).
    Serve {
        /// Run MCP server over stdio (JSON-RPC). Project config is loaded from cwd.
        #[arg(long)]
        mcp: bool,
    },
    /// Interact with external MCP servers listed in `providers.toml` under `[mcp_client]`.
    Mcp {
        #[command(subcommand)]
        sub: McpCommand,
    },
    /// Diff working tree vs `HEAD` with optional semantic overlay from `cantrik index`.
    Diff {
        #[arg(long)]
        staged: bool,
        /// Skip symbol/caller mapping from `.cantrik/index/ast/` (unified diff only).
        #[arg(long)]
        text_only: bool,
        #[arg(long)]
        conflicts: bool,
    },
    /// Write `.cantrik/handoff-YYYY-MM-DD.md` from session decisions and summary.
    Handoff {
        #[arg(long, value_name = "TEXT")]
        message: Option<String>,
    },
    /// Export lightweight context JSON (project rules, cantrik.toml, skill paths, message tail).
    Export {
        #[arg(short, long, value_name = "PATH")]
        output: PathBuf,
    },
    /// Import a context bundle written by `cantrik export`.
    Import {
        #[arg(short, long, value_name = "PATH")]
        input: PathBuf,
        /// Append one assistant message summarizing imported message tail.
        #[arg(long)]
        seed_session: bool,
    },
    /// Session replay log: export JSON or print timeline (no tool re-execution).
    Replay {
        #[command(subcommand)]
        sub: ReplayCommand,
    },
    /// Feature branch and AI-assisted commit (Sprint 16). Does not replace read-only `cantrik git`.
    Workspace {
        #[command(subcommand)]
        sub: WorkspaceCommand,
    },
    /// Open a GitHub pull request via `gh pr create` (Sprint 16).
    Pr {
        #[command(subcommand)]
        sub: PrCommand,
    },
    /// LLM review of staged diff (default) or full worktree vs HEAD (`--worktree`).
    Review {
        #[arg(long)]
        worktree: bool,
        /// On LLM failure exit 0 (for optional git hooks).
        #[arg(long)]
        soft: bool,
    },
    /// Git blame + log for a file; optional LLM narrative (`--why`, Sprint 17).
    Explain {
        /// Source file (required; repository path).
        #[arg(value_name = "FILE")]
        path: PathBuf,
        /// Ask the configured LLM to synthesize evolution / intent from blame + log.
        #[arg(long)]
        why: bool,
        /// Focus `git blame` window starting at this 1-based line.
        #[arg(long, value_name = "N")]
        line: Option<u32>,
    },
    /// Draft architecture / ADR stubs from repo context (Sprint 17); LLM required.
    Teach {
        /// Write `ARCHITECTURE.md` here instead of printing to stdout.
        #[arg(long, value_name = "DIR")]
        output_dir: Option<PathBuf>,
        /// `markdown` (default) or `wiki` (Obsidian-style frontmatter + `[[wikilinks]]`).
        #[arg(long, value_enum, default_value_t = TeachFormatArg::Markdown)]
        format: TeachFormatArg,
    },
    /// Why a crate is in the tree (`cargo tree -i`, Sprint 17).
    Why {
        /// Dependency package name as in Cargo.toml.
        #[arg(value_name = "CRATE")]
        crate_name: String,
        /// Optional LLM summary paragraph.
        #[arg(long)]
        synthesize: bool,
    },
    /// Suggested upgrade priorities from lockfile + shallow tree (LLM; no `cargo update`, Sprint 17).
    Upgrade,
    /// Run `cargo audit` or `[intelligence].audit_command` (Sprint 17).
    Audit,
    /// Issue URL: prints suggested workflow; full auto-fix deferred (Sprint 16).
    Fix {
        #[arg(value_name = "ISSUE_URL")]
        issue_url: String,
    },
    /// Web search / fetch with explicit `--approve` (Sprint 16, PRD §4.13).
    Web {
        #[command(subcommand)]
        sub: WebCommand,
    },
    /// Mermaid diagram from index, layout, or `cargo tree` (Sprint 18, PRD §4.17).
    Visualize {
        #[arg(value_enum, default_value_t = VisualizeCliKind::Callgraph)]
        mode: VisualizeCliKind,
        /// Write `.mmd` (or path) instead of stdout.
        #[arg(short, long, value_name = "PATH")]
        output: Option<PathBuf>,
    },
    /// Transcribe audio via Ollama when `[ui].voice_enabled` (or use `--raw-text` for tests).
    Listen {
        /// Audio file to send to `{ollama}/api/transcribe`.
        #[arg(long, value_name = "FILE")]
        file: Option<PathBuf>,
        /// Skip STT and run `ask` with this text (for testing without audio).
        #[arg(long, value_name = "TEXT")]
        raw_text: Option<String>,
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
pub enum SkillCommand {
    /// Copy files from `~/.local/share/cantrik/skill-registry/<name>/` into `.cantrik/`.
    Install {
        #[arg(value_name = "NAME")]
        name: String,
    },
    /// List package names in the local registry.
    List,
    /// Remove installed files tracked in `.cantrik/installed-skills.toml`.
    Remove {
        #[arg(value_name = "NAME")]
        name: String,
    },
    /// Re-copy package files from the registry.
    Update {
        #[arg(value_name = "NAME")]
        name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum MacroCommand {
    /// Start recording steps for a named macro.
    Record {
        #[arg(value_name = "LABEL")]
        label: String,
    },
    /// Append one step (full argv) to the active recording.
    Add {
        #[arg(required = true, trailing_var_arg = true, value_name = "ARGS")]
        args: Vec<String>,
    },
    /// Save recording to `.cantrik/macros/<label>.json`.
    Stop,
    /// Run a saved macro by label.
    Run {
        #[arg(value_name = "LABEL")]
        label: String,
    },
    /// List saved macro labels.
    List,
}

#[derive(Debug, Subcommand)]
pub enum McpCommand {
    /// Invoke a tool on a named `[[mcp_client.servers]]` entry (spawns stdio child).
    Call {
        #[arg(value_name = "SERVER")]
        server: String,
        #[arg(value_name = "TOOL")]
        tool: String,
        /// JSON object passed as tool arguments (default `{}`).
        #[arg(long, default_value = "{}")]
        json: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ReplayCommand {
    /// Export recent session messages to JSON (schema version 1).
    Export {
        #[arg(short, long, value_name = "PATH")]
        output: PathBuf,
    },
    /// Print timeline from a replay JSON file (dry replay).
    Play {
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
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

/// Output style for `cantrik teach` (Sprint 17).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum TeachFormatArg {
    #[default]
    Markdown,
    Wiki,
}

/// `cantrik visualize` mode (Sprint 18, PRD §4.17).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum VisualizeCliKind {
    #[default]
    Callgraph,
    Architecture,
    Dependencies,
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
