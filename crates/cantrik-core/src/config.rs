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
        }
    }
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
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn project_config_overrides_global_config() {
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
}
