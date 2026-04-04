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
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct UiConfig {
    pub language: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct LlmConfig {
    pub provider: Option<String>,
    pub model: Option<String>,
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
            },
            llm: LlmConfig {
                provider: override_config.llm.provider.or(self.llm.provider),
                model: override_config.llm.model.or(self.llm.model),
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
}
