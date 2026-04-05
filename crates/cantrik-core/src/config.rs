use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct AppConfig {
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub index: IndexConfig,
    #[serde(default)]
    pub memory: MemoryConfig,
    #[serde(default)]
    pub sandbox: SandboxConfig,
    #[serde(default)]
    pub guardrails: GuardrailsConfig,
    #[serde(default)]
    pub audit: AuditTrackConfig,
    #[serde(default)]
    pub planning: PlanningConfig,
    #[serde(default)]
    pub agents: AgentsConfig,
    #[serde(default)]
    pub background: BackgroundConfig,
    #[serde(default)]
    pub skills: SkillsConfig,
    /// Semantic diff, handoff, export/import, replay (Sprint 15).
    #[serde(default)]
    pub collab: CollabConfig,
    /// Git branch/PR workflow and web research caps (Sprint 16).
    #[serde(default)]
    pub git_workflow: GitWorkflowConfig,
    /// Explain, teach, dependency intel (Sprint 17, PRD §4.20, §4.24–4.25).
    #[serde(default)]
    pub intelligence: IntelligenceConfig,
    /// Optional extra workspace roots for session fingerprint (Sprint 18 follow-up, monorepo).
    #[serde(default)]
    pub workspace: WorkspaceConfig,
    /// `cantrik exec --remote` target (hybrid SSH MVP, RFC `docs/rfc-hybrid-ssh-executor.md`).
    #[serde(default)]
    pub remote_exec: RemoteExecConfig,
}

/// SSH destination for remote command preview / execution (`cantrik exec --remote`).
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct RemoteExecConfig {
    /// Remote hostname or `user@host` form is **not** supported here — use `user` + `host` separately.
    pub host: Option<String>,
    pub user: Option<String>,
    /// Passed to `ssh -i` when set.
    pub identity_file: Option<String>,
    /// Extra `ssh` argv inserted before the destination (e.g. `-p`, `2222`).
    #[serde(default)]
    pub extra_ssh_args: Vec<String>,
}

/// `ssh` destination string (`user@host` or bare `host` when user omitted).
pub fn remote_ssh_destination(c: &RemoteExecConfig) -> Option<String> {
    let host = c.host.as_deref().map(str::trim).filter(|s| !s.is_empty())?;
    match c.user.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(u) => Some(format!("{u}@{host}")),
        None => Some(host.to_string()),
    }
}

/// Extra directories merged into the session project fingerprint when non-empty (Sprint 18 follow-up).
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct WorkspaceConfig {
    /// Paths relative to the primary project directory or absolute; invalid entries are skipped with a warning.
    #[serde(default)]
    pub extra_roots: Vec<String>,
}

/// Intelligence tools limits and audit command override (Sprint 17).
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct IntelligenceConfig {
    /// Max lines included from `git blame` output (default 200).
    pub explain_max_blame_lines: Option<u32>,
    /// Max files scanned for `teach` context (default 64).
    pub teach_max_files_scanned: Option<u32>,
    /// Override audit argv (default: `cargo audit`).
    #[serde(default)]
    pub audit_command: Option<Vec<String>>,
}

/// Git-native workflow + web tool limits (Sprint 16, PRD §4.11, §4.13).
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct GitWorkflowConfig {
    /// Prefix for `workspace branch start` (default `feature/cantrik-`).
    pub branch_prefix: Option<String>,
    /// `auto` | `gh` | `none` — PR creation backend (default `auto`).
    pub pr_provider: Option<String>,
    pub web_search_max_results: Option<u32>,
    pub web_fetch_max_bytes: Option<u64>,
}

/// Collaboration / semantic diff limits (Sprint 15, PRD §4.8, §4.23, §4.27).
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct CollabConfig {
    /// Cap files processed in `cantrik diff` (default 200).
    pub max_files_in_report: Option<u32>,
    /// Tail message count in `cantrik export` / `cantrik replay export` (default 50).
    pub replay_tail_messages: Option<u32>,
}

/// Skill files under `.cantrik/skills/` + injection policy (Sprint 13, PRD §7).
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct SkillsConfig {
    /// When false, no skill files are injected (rules.md still applies if present).
    pub auto_inject: Option<bool>,
    /// Max total characters for all selected skill bodies (excluding headers).
    pub max_total_chars: Option<u64>,
    /// Max number of skill files to include after scoring.
    pub max_files: Option<u32>,
    /// If non-empty after merge, only these basenames (e.g. `backend.md`) are considered.
    #[serde(default)]
    pub files: Vec<String>,
}

/// PRD §6 Enhancement — optional Javanese-flavoured tone hints (Sprint 18).
#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CulturalWisdomLevel {
    #[default]
    Off,
    Light,
    Full,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct UiConfig {
    pub language: Option<String>,
    /// `off` | `light` | `full` — injects a short style block into session prompts.
    pub cultural_wisdom: Option<CulturalWisdomLevel>,
    /// Gate `cantrik listen` and optional TTS after background jobs (Sprint 18, PRD §4.26).
    pub voice_enabled: Option<bool>,
    /// Wider REPL layout: assistant + preview pane (Sprint 18, PRD §6).
    pub tui_split_pane: Option<bool>,
    /// Model name for Ollama `/api/transcribe` when available (default `whisper`).
    pub transcription_model: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct LlmConfig {
    pub provider: Option<String>,
    pub model: Option<String>,
    /// When true (or `CANTRIK_OFFLINE=1`), only local Ollama on a loopback base URL is used (Sprint 19).
    pub offline: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct IndexConfig {
    /// Ollama embedding model (default in code: `nomic-embed-text`).
    pub vector_model: Option<String>,
    /// Ollama HTTP base URL; empty uses `OLLAMA_HOST` / `providers.toml` / `http://127.0.0.1:11434`.
    pub ollama_base: Option<String>,
}

/// Sandbox level for `run_command` (PRD §4.7).
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum SandboxLevel {
    /// No isolation (opt-in only).
    None,
    /// Default — bubblewrap on Linux, sandbox-exec on macOS when available.
    #[default]
    Restricted,
    /// Not implemented in Sprint 8.
    Container,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
pub struct SandboxConfig {
    /// When `None`, defaults to [`SandboxLevel::Restricted`] at runtime.
    #[serde(default)]
    pub level: Option<SandboxLevel>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct GuardrailsConfig {
    /// Tool ids (e.g. `run_command`, `write_file`) denied regardless of other lists.
    #[serde(default)]
    pub forbidden: Vec<String>,
    /// Tools that require explicit approval in agent mode (CLI still uses `--approve` where applicable).
    #[serde(default)]
    pub require_approval: Vec<String>,
    /// Tools that may run without a second prompt when autonomy allows (see docs).
    #[serde(default)]
    pub auto_approve: Vec<String>,
}

/// Multi-agent orchestration (Sprint 11, PRD §4.2).
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct AgentsConfig {
    /// Max nested spawn depth (default 3); MVP entry uses depth 0 only.
    pub max_spawn_depth: Option<u8>,
    /// Concurrent sub-agent LLM calls (default 4).
    pub max_parallel_subagents: Option<u32>,
    /// Truncate sub-agent summary for synthesis prompt (default 2000).
    pub subagent_summary_max_chars: Option<u32>,
}

/// Background daemon jobs + approval notifications (Sprint 12, PRD §4.3).
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct BackgroundConfig {
    /// POST JSON `{ event, job_id, hint }` when a job enters `waiting_approval`.
    pub webhook_url: Option<String>,
    /// When true, try desktop notification (`notify-send` / `osascript`).
    pub desktop_notify: Option<bool>,
    /// Touch/write this path with job id when approval is needed (default: share dir flag file).
    pub approval_flag_path: Option<String>,
    /// LLM rounds per job before `completed` (each round may pause at `waiting_approval`).
    pub max_llm_rounds: Option<u32>,
}

/// Planning / experiment loop (Sprint 10).
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct PlanningConfig {
    /// Failures on the same step before stuck escalation (default 3).
    pub stuck_threshold_attempts: Option<u32>,
    /// Re-plan generations before forced escalation (default 2).
    pub max_replan_cycles: Option<u32>,
    /// argv for experiment verification (default `cargo test`).
    pub experiment_test_command: Option<Vec<String>>,
}

/// Audit log / provenance toggles (Sprint 9).
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct AuditTrackConfig {
    /// `file` (default when unset) appends `.cantrik/provenance.jsonl` on write; `off` disables.
    #[serde(default)]
    pub provenance: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct MemoryConfig {
    /// Summarize when sum of chars in "old" messages exceeds this (excluding tail).
    pub summarize_threshold_chars: Option<u64>,
    /// Hard cap on assembled prompt size sent to the LLM.
    pub max_context_chars: Option<u64>,
    /// Max bytes for `read_file` tool / CLI.
    pub max_file_read_bytes: Option<u64>,
    /// Verbatim tail message count preserved when summarizing.
    pub context_tail_messages: Option<u32>,
    /// When true, record explicit CLI approvals and inject a short summary into session prompts (Sprint 19, PRD §4.15).
    pub adaptive_begawan: Option<bool>,
    /// Max characters for the adaptive memory block in prompts (default 900).
    pub adaptive_begawan_max_chars: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigPaths {
    pub global: PathBuf,
    pub project: PathBuf,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file at {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse TOML config at {path}: {source}")]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
}

impl AppConfig {
    fn merge(self, override_config: AppConfig) -> AppConfig {
        AppConfig {
            ui: UiConfig {
                language: override_config.ui.language.or(self.ui.language),
                cultural_wisdom: override_config
                    .ui
                    .cultural_wisdom
                    .or(self.ui.cultural_wisdom),
                voice_enabled: override_config.ui.voice_enabled.or(self.ui.voice_enabled),
                tui_split_pane: override_config.ui.tui_split_pane.or(self.ui.tui_split_pane),
                transcription_model: override_config
                    .ui
                    .transcription_model
                    .clone()
                    .or_else(|| self.ui.transcription_model.clone()),
            },
            llm: LlmConfig {
                provider: override_config.llm.provider.or(self.llm.provider),
                model: override_config.llm.model.or(self.llm.model),
                offline: override_config.llm.offline.or(self.llm.offline),
            },
            index: IndexConfig {
                vector_model: override_config
                    .index
                    .vector_model
                    .or(self.index.vector_model),
                ollama_base: override_config.index.ollama_base.or(self.index.ollama_base),
            },
            memory: MemoryConfig {
                summarize_threshold_chars: override_config
                    .memory
                    .summarize_threshold_chars
                    .or(self.memory.summarize_threshold_chars),
                max_context_chars: override_config
                    .memory
                    .max_context_chars
                    .or(self.memory.max_context_chars),
                max_file_read_bytes: override_config
                    .memory
                    .max_file_read_bytes
                    .or(self.memory.max_file_read_bytes),
                context_tail_messages: override_config
                    .memory
                    .context_tail_messages
                    .or(self.memory.context_tail_messages),
                adaptive_begawan: override_config
                    .memory
                    .adaptive_begawan
                    .or(self.memory.adaptive_begawan),
                adaptive_begawan_max_chars: override_config
                    .memory
                    .adaptive_begawan_max_chars
                    .or(self.memory.adaptive_begawan_max_chars),
            },
            sandbox: SandboxConfig {
                level: override_config.sandbox.level.or(self.sandbox.level),
            },
            guardrails: GuardrailsConfig {
                forbidden: merge_str_lists(
                    &self.guardrails.forbidden,
                    &override_config.guardrails.forbidden,
                ),
                require_approval: merge_str_lists(
                    &self.guardrails.require_approval,
                    &override_config.guardrails.require_approval,
                ),
                auto_approve: merge_str_lists(
                    &self.guardrails.auto_approve,
                    &override_config.guardrails.auto_approve,
                ),
            },
            audit: AuditTrackConfig {
                provenance: override_config.audit.provenance.or(self.audit.provenance),
            },
            planning: PlanningConfig {
                stuck_threshold_attempts: override_config
                    .planning
                    .stuck_threshold_attempts
                    .or(self.planning.stuck_threshold_attempts),
                max_replan_cycles: override_config
                    .planning
                    .max_replan_cycles
                    .or(self.planning.max_replan_cycles),
                experiment_test_command: override_config
                    .planning
                    .experiment_test_command
                    .clone()
                    .or_else(|| self.planning.experiment_test_command.clone()),
            },
            agents: AgentsConfig {
                max_spawn_depth: override_config
                    .agents
                    .max_spawn_depth
                    .or(self.agents.max_spawn_depth),
                max_parallel_subagents: override_config
                    .agents
                    .max_parallel_subagents
                    .or(self.agents.max_parallel_subagents),
                subagent_summary_max_chars: override_config
                    .agents
                    .subagent_summary_max_chars
                    .or(self.agents.subagent_summary_max_chars),
            },
            background: BackgroundConfig {
                webhook_url: override_config
                    .background
                    .webhook_url
                    .clone()
                    .or_else(|| self.background.webhook_url.clone()),
                desktop_notify: override_config
                    .background
                    .desktop_notify
                    .or(self.background.desktop_notify),
                approval_flag_path: override_config
                    .background
                    .approval_flag_path
                    .clone()
                    .or_else(|| self.background.approval_flag_path.clone()),
                max_llm_rounds: override_config
                    .background
                    .max_llm_rounds
                    .or(self.background.max_llm_rounds),
            },
            skills: SkillsConfig {
                auto_inject: override_config
                    .skills
                    .auto_inject
                    .or(self.skills.auto_inject),
                max_total_chars: override_config
                    .skills
                    .max_total_chars
                    .or(self.skills.max_total_chars),
                max_files: override_config.skills.max_files.or(self.skills.max_files),
                files: merge_str_lists(&self.skills.files, &override_config.skills.files),
            },
            collab: CollabConfig {
                max_files_in_report: override_config
                    .collab
                    .max_files_in_report
                    .or(self.collab.max_files_in_report),
                replay_tail_messages: override_config
                    .collab
                    .replay_tail_messages
                    .or(self.collab.replay_tail_messages),
            },
            git_workflow: GitWorkflowConfig {
                branch_prefix: override_config
                    .git_workflow
                    .branch_prefix
                    .clone()
                    .or_else(|| self.git_workflow.branch_prefix.clone()),
                pr_provider: override_config
                    .git_workflow
                    .pr_provider
                    .clone()
                    .or_else(|| self.git_workflow.pr_provider.clone()),
                web_search_max_results: override_config
                    .git_workflow
                    .web_search_max_results
                    .or(self.git_workflow.web_search_max_results),
                web_fetch_max_bytes: override_config
                    .git_workflow
                    .web_fetch_max_bytes
                    .or(self.git_workflow.web_fetch_max_bytes),
            },
            intelligence: IntelligenceConfig {
                explain_max_blame_lines: override_config
                    .intelligence
                    .explain_max_blame_lines
                    .or(self.intelligence.explain_max_blame_lines),
                teach_max_files_scanned: override_config
                    .intelligence
                    .teach_max_files_scanned
                    .or(self.intelligence.teach_max_files_scanned),
                audit_command: override_config
                    .intelligence
                    .audit_command
                    .clone()
                    .or_else(|| self.intelligence.audit_command.clone()),
            },
            workspace: WorkspaceConfig {
                extra_roots: merge_str_lists(
                    &self.workspace.extra_roots,
                    &override_config.workspace.extra_roots,
                ),
            },
            remote_exec: RemoteExecConfig {
                host: override_config
                    .remote_exec
                    .host
                    .clone()
                    .or_else(|| self.remote_exec.host.clone()),
                user: override_config
                    .remote_exec
                    .user
                    .clone()
                    .or_else(|| self.remote_exec.user.clone()),
                identity_file: override_config
                    .remote_exec
                    .identity_file
                    .clone()
                    .or_else(|| self.remote_exec.identity_file.clone()),
                extra_ssh_args: if !override_config.remote_exec.extra_ssh_args.is_empty() {
                    override_config.remote_exec.extra_ssh_args.clone()
                } else {
                    self.remote_exec.extra_ssh_args.clone()
                },
            },
        }
    }
}

pub fn effective_stuck_threshold(c: &PlanningConfig) -> u32 {
    c.stuck_threshold_attempts.unwrap_or(3)
}

pub fn effective_max_replan_cycles(c: &PlanningConfig) -> u32 {
    c.max_replan_cycles.unwrap_or(2)
}

pub fn effective_experiment_test_command(c: &PlanningConfig) -> Vec<String> {
    c.experiment_test_command
        .clone()
        .unwrap_or_else(|| vec!["cargo".to_string(), "test".to_string()])
}

pub fn effective_max_spawn_depth(c: &AgentsConfig) -> u8 {
    c.max_spawn_depth.unwrap_or(3)
}

pub fn effective_max_parallel_subagents(c: &AgentsConfig) -> usize {
    c.max_parallel_subagents.unwrap_or(4).max(1) as usize
}

pub fn effective_subagent_summary_max_chars(c: &AgentsConfig) -> usize {
    c.subagent_summary_max_chars.unwrap_or(2000).max(256) as usize
}

pub fn effective_background_max_llm_rounds(c: &BackgroundConfig) -> u32 {
    // Default 2: first round ends in `waiting_approval` so notification + resume path is exercised.
    c.max_llm_rounds.unwrap_or(2).max(1)
}

/// Desktop notify: explicit config wins; otherwise enabled on Linux/macOS.
pub fn effective_background_desktop_notify(c: &BackgroundConfig) -> bool {
    if let Some(v) = c.desktop_notify {
        return v;
    }
    cfg!(any(target_os = "linux", target_os = "macos"))
}

pub fn effective_skills_auto_inject(c: &SkillsConfig) -> bool {
    c.auto_inject.unwrap_or(true)
}

pub fn effective_skills_max_total_chars(c: &SkillsConfig) -> u64 {
    c.max_total_chars.unwrap_or(32_000).max(512)
}

pub fn effective_skills_max_files(c: &SkillsConfig) -> u32 {
    c.max_files.unwrap_or(4).max(1)
}

pub fn effective_cultural_wisdom(c: &UiConfig) -> CulturalWisdomLevel {
    c.cultural_wisdom.unwrap_or(CulturalWisdomLevel::Off)
}

pub fn effective_voice_enabled(c: &UiConfig) -> bool {
    c.voice_enabled.unwrap_or(false)
}

pub fn effective_tui_split_pane(c: &UiConfig) -> bool {
    c.tui_split_pane.unwrap_or(false)
}

pub fn effective_transcription_model(c: &UiConfig) -> String {
    c.transcription_model
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "whisper".to_string())
}

pub fn effective_collab_max_files_in_report(c: &CollabConfig) -> usize {
    c.max_files_in_report.unwrap_or(200).max(1) as usize
}

pub fn effective_collab_replay_tail_messages(c: &CollabConfig) -> i64 {
    i64::from(c.replay_tail_messages.unwrap_or(50).max(1))
}

pub fn effective_git_branch_prefix(c: &GitWorkflowConfig) -> String {
    c.branch_prefix
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "feature/cantrik-".to_string())
}

/// Returns `gh`, `none`, or `auto` (try `gh` when origin looks like GitHub).
pub fn effective_pr_provider(c: &GitWorkflowConfig) -> String {
    c.pr_provider
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "auto".to_string())
        .to_ascii_lowercase()
}

pub fn effective_web_search_max_results(c: &GitWorkflowConfig) -> usize {
    c.web_search_max_results.unwrap_or(5).max(1) as usize
}

pub fn effective_web_fetch_max_bytes(c: &GitWorkflowConfig, fallback: u64) -> u64 {
    c.web_fetch_max_bytes.unwrap_or(fallback).max(1024)
}

pub fn effective_explain_max_blame_lines(c: &IntelligenceConfig) -> usize {
    c.explain_max_blame_lines.unwrap_or(200).max(1) as usize
}

pub fn effective_teach_max_files_scanned(c: &IntelligenceConfig) -> usize {
    c.teach_max_files_scanned.unwrap_or(64).max(8) as usize
}

pub fn effective_audit_command(c: &IntelligenceConfig) -> Vec<String> {
    c.audit_command
        .clone()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| vec!["cargo".into(), "audit".into()])
}

/// Adaptive Begawan: off unless `[memory].adaptive_begawan = true`.
pub fn effective_adaptive_begawan(c: &MemoryConfig) -> bool {
    c.adaptive_begawan.unwrap_or(false)
}

pub fn effective_adaptive_begawan_max_chars(c: &MemoryConfig) -> usize {
    c.adaptive_begawan_max_chars.unwrap_or(900).max(200) as usize
}

/// Air-gapped / offline LLM: enabled when `[llm].offline = true` or `CANTRIK_OFFLINE` is `1`/`true`/`yes`.
pub fn effective_llm_offline(c: &LlmConfig) -> bool {
    if let Ok(v) = env::var("CANTRIK_OFFLINE") {
        let s = v.trim().to_ascii_lowercase();
        if matches!(s.as_str(), "1" | "true" | "yes" | "on") {
            return true;
        }
    }
    c.offline.unwrap_or(false)
}

/// When offline mode is on, outbound HTTP CLI tools (`fetch`, `web`, agent `web_fetch` / `browse_page`, …)
/// and background webhooks should refuse or skip (enterprise audit).
pub fn offline_blocks_outbound_http(app: &AppConfig) -> bool {
    effective_llm_offline(&app.llm)
}

/// User-visible reason when [`offline_blocks_outbound_http`] is true.
pub fn offline_http_blocked_message() -> &'static str {
    "refused: offline/air-gapped mode is on ([llm].offline or CANTRIK_OFFLINE); outbound HTTP is disabled"
}

/// Canonical, sorted, deduplicated extra roots from config (excludes the primary cwd).
pub fn effective_workspace_extra_roots(primary: &Path, app: &AppConfig) -> Vec<PathBuf> {
    use std::collections::HashSet;

    let mut seen: HashSet<PathBuf> = HashSet::new();
    let mut out: Vec<PathBuf> = Vec::new();
    for raw in &app.workspace.extra_roots {
        let p = PathBuf::from(raw);
        let joined = if p.is_absolute() { p } else { primary.join(p) };
        match fs::canonicalize(&joined) {
            Ok(c) => {
                if seen.insert(c.clone()) {
                    out.push(c);
                }
            }
            Err(e) => {
                eprintln!("cantrik: [workspace].extra_roots: skip {joined:?}: {e}");
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Whether to append provenance JSONL on successful writes.
pub fn provenance_file_enabled(c: &AuditTrackConfig) -> bool {
    !matches!(c.provenance.as_deref(), Some("off"))
}

/// Resolved sandbox level (`None` in config → restricted).
pub fn effective_sandbox_level(c: &SandboxConfig) -> SandboxLevel {
    c.level.unwrap_or(SandboxLevel::Restricted)
}

fn merge_str_lists(base: &[String], project: &[String]) -> Vec<String> {
    if project.is_empty() {
        return base.to_vec();
    }
    let mut out: Vec<String> = base.to_vec();
    for s in project {
        if !out.iter().any(|x| x == s) {
            out.push(s.clone());
        }
    }
    out
}

pub fn resolve_config_paths(cwd: &Path) -> ConfigPaths {
    let home = env::var_os("HOME").map(PathBuf::from).unwrap_or_default();

    let global = home.join(".config").join("cantrik").join("config.toml");
    let project = cwd.join(".cantrik").join("cantrik.toml");

    ConfigPaths { global, project }
}

pub fn load_merged_config(cwd: &Path) -> Result<AppConfig, ConfigError> {
    let paths = resolve_config_paths(cwd);

    let global = read_if_exists(&paths.global)?;
    let project = read_if_exists(&paths.project)?;

    Ok(global.merge(project))
}

fn read_if_exists(path: &Path) -> Result<AppConfig, ConfigError> {
    if !path.exists() {
        return Ok(AppConfig::default());
    }

    let contents = fs::read_to_string(path).map_err(|source| ConfigError::Read {
        path: path.to_path_buf(),
        source,
    })?;

    toml::from_str(&contents).map_err(|source| ConfigError::Parse {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Tests that mutate `HOME` must not run in parallel.
    static CONFIG_TEST_HOME_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn project_config_overrides_global_config() {
        let _home_lock = CONFIG_TEST_HOME_LOCK.lock().expect("home test lock");
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be valid")
            .as_nanos();

        let base = env::temp_dir().join(format!("cantrik-config-test-{unique}"));
        let global_dir = base.join("home/.config/cantrik");
        let project_dir = base.join("project/.cantrik");

        fs::create_dir_all(&global_dir).expect("global dir should be created");
        fs::create_dir_all(&project_dir).expect("project dir should be created");

        fs::write(
            global_dir.join("config.toml"),
            "[ui]\nlanguage = \"id\"\n[llm]\nprovider = \"anthropic\"\n",
        )
        .expect("global config should be written");

        fs::write(
            project_dir.join("cantrik.toml"),
            "[llm]\nprovider = \"ollama\"\nmodel = \"llama3\"\n",
        )
        .expect("project config should be written");

        let old_home = env::var_os("HOME");
        // HOME is overridden in test to validate global config discovery logic.
        unsafe {
            env::set_var("HOME", base.join("home"));
        }

        let config = load_merged_config(&base.join("project"));

        match old_home {
            Some(value) => unsafe {
                env::set_var("HOME", value);
            },
            None => unsafe {
                env::remove_var("HOME");
            },
        }

        fs::remove_dir_all(&base).expect("temp dirs should be removable");

        let config = config.expect("config should load");
        assert_eq!(config.ui.language.as_deref(), Some("id"));
        assert_eq!(config.llm.provider.as_deref(), Some("ollama"));
        assert_eq!(config.llm.model.as_deref(), Some("llama3"));
    }

    #[test]
    fn guardrails_lists_merge_project_into_global() {
        let _home_lock = CONFIG_TEST_HOME_LOCK.lock().expect("home test lock");
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be valid")
            .as_nanos();

        let base = env::temp_dir().join(format!("cantrik-guard-{unique}"));
        let global_dir = base.join("home/.config/cantrik");
        let project_dir = base.join("project/.cantrik");

        fs::create_dir_all(&global_dir).expect("global dir");
        fs::create_dir_all(&project_dir).expect("project dir");

        fs::write(
            global_dir.join("config.toml"),
            "[guardrails]\nforbidden = [\"write_file\"]\n",
        )
        .expect("global");

        fs::write(
            project_dir.join("cantrik.toml"),
            "[guardrails]\nforbidden = [\"run_command\"]\n",
        )
        .expect("project");

        let old_home = env::var_os("HOME");
        unsafe {
            env::set_var("HOME", base.join("home"));
        }

        let config = load_merged_config(&base.join("project")).expect("load");

        match old_home {
            Some(value) => unsafe {
                env::set_var("HOME", value);
            },
            None => unsafe {
                env::remove_var("HOME");
            },
        }

        fs::remove_dir_all(&base).expect("cleanup");

        assert!(config.guardrails.forbidden.contains(&"write_file".into()));
        assert!(config.guardrails.forbidden.contains(&"run_command".into()));
    }

    #[test]
    fn intelligence_section_merge_and_effective_helpers() {
        let base = AppConfig::default();
        let override_config = AppConfig {
            intelligence: IntelligenceConfig {
                explain_max_blame_lines: Some(50),
                teach_max_files_scanned: Some(16),
                audit_command: Some(vec!["custom-audit".into(), "--json".into()]),
            },
            ..Default::default()
        };
        let merged = base.merge(override_config);
        assert_eq!(merged.intelligence.explain_max_blame_lines, Some(50));
        assert_eq!(effective_explain_max_blame_lines(&merged.intelligence), 50);
        assert_eq!(effective_teach_max_files_scanned(&merged.intelligence), 16);
        assert_eq!(
            effective_audit_command(&merged.intelligence),
            vec!["custom-audit", "--json"]
        );
        let def = IntelligenceConfig::default();
        assert_eq!(effective_explain_max_blame_lines(&def), 200);
        assert_eq!(effective_audit_command(&def), vec!["cargo", "audit"]);
    }

    #[test]
    fn ui_ux_fields_merge_and_effective() {
        let base = AppConfig::default();
        let over = AppConfig {
            ui: UiConfig {
                cultural_wisdom: Some(CulturalWisdomLevel::Light),
                voice_enabled: Some(true),
                tui_split_pane: Some(true),
                transcription_model: Some("large-v3".into()),
                ..Default::default()
            },
            ..Default::default()
        };
        let m = base.merge(over);
        assert_eq!(effective_cultural_wisdom(&m.ui), CulturalWisdomLevel::Light);
        assert!(effective_voice_enabled(&m.ui));
        assert!(effective_tui_split_pane(&m.ui));
        assert_eq!(effective_transcription_model(&m.ui), "large-v3");
    }

    #[test]
    fn workspace_extra_roots_merge_global_and_project() {
        let base = AppConfig {
            workspace: WorkspaceConfig {
                extra_roots: vec!["../lib-a".into()],
            },
            ..Default::default()
        };
        let over = AppConfig {
            workspace: WorkspaceConfig {
                extra_roots: vec!["../lib-b".into()],
            },
            ..Default::default()
        };
        let m = base.merge(over);
        assert_eq!(m.workspace.extra_roots.len(), 2);
        assert!(m.workspace.extra_roots.contains(&"../lib-a".into()));
        assert!(m.workspace.extra_roots.contains(&"../lib-b".into()));
    }
}
