//! Load `~/.config/cantrik/providers.toml` (PRD) and resolve credentials.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

/// Well-known provider ids in config (case-insensitive when parsing routes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderKind {
    Anthropic,
    Gemini,
    Ollama,
    /// OpenAI (`/v1/chat/completions`).
    OpenAi,
    /// Azure OpenAI (deployment in URL).
    AzureOpenAi,
    /// OpenRouter (OpenAI-compatible API).
    OpenRouter,
    /// Groq (`/openai/v1/chat/completions`).
    Groq,
}

impl ProviderKind {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "anthropic" | "claude" => Some(Self::Anthropic),
            "gemini" | "google" => Some(Self::Gemini),
            "ollama" => Some(Self::Ollama),
            "openai" | "openai-api" => Some(Self::OpenAi),
            "azure" | "azure-openai" | "openai-azure" => Some(Self::AzureOpenAi),
            "openrouter" => Some(Self::OpenRouter),
            "groq" => Some(Self::Groq),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Anthropic => "anthropic",
            Self::Gemini => "gemini",
            Self::Ollama => "ollama",
            Self::OpenAi => "openai",
            Self::AzureOpenAi => "azure",
            Self::OpenRouter => "openrouter",
            Self::Groq => "groq",
        }
    }

    /// All kinds shown in `cantrik doctor` (order stable).
    pub const ALL: [ProviderKind; 7] = [
        ProviderKind::Anthropic,
        ProviderKind::Gemini,
        ProviderKind::OpenAi,
        ProviderKind::AzureOpenAi,
        ProviderKind::OpenRouter,
        ProviderKind::Groq,
        ProviderKind::Ollama,
    ];
}

/// Resolved model id for one HTTP call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderTarget {
    pub kind: ProviderKind,
    pub model: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProvidersToml {
    pub providers: ProviderSections,
    #[serde(default)]
    pub routing: Option<RoutingSection>,
    #[serde(default)]
    pub mcp_client: Option<McpClientSection>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProviderSections {
    pub anthropic: Option<AnthropicSection>,
    pub gemini: Option<GeminiSection>,
    pub ollama: Option<OllamaSection>,
    pub openai: Option<OpenAiSection>,
    pub azure: Option<AzureOpenAiSection>,
    pub openrouter: Option<OpenRouterSection>,
    pub groq: Option<GroqSection>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OpenAiSection {
    pub api_key: Option<String>,
    #[serde(default)]
    pub default_model: Option<String>,
    /// Base URL including `/v1`, e.g. `https://api.openai.com/v1`.
    #[serde(default)]
    pub base_url: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AzureOpenAiSection {
    pub api_key: Option<String>,
    /// Resource endpoint, e.g. `https://myresource.openai.azure.com`.
    pub endpoint: String,
    /// Default deployment name when routing omits `/model`.
    #[serde(default)]
    pub default_deployment: Option<String>,
    #[serde(default)]
    pub api_version: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OpenRouterSection {
    pub api_key: Option<String>,
    #[serde(default)]
    pub default_model: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GroqSection {
    pub api_key: Option<String>,
    #[serde(default)]
    pub default_model: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AnthropicSection {
    pub api_key: Option<String>,
    #[serde(default)]
    pub default_model: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GeminiSection {
    pub api_key: Option<String>,
    #[serde(default)]
    pub default_model: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OllamaSection {
    #[serde(default = "default_ollama_base")]
    pub base_url: String,
    #[serde(default)]
    pub default_model: Option<String>,
    #[serde(default)]
    pub _embed_model: Option<String>,
}

fn default_ollama_base() -> String {
    "http://127.0.0.1:11434".to_string()
}

/// Optional per-tier route strings (`provider/model` or `provider`), used when `auto_route = true`.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct RoutingThresholds {
    #[serde(default)]
    pub simple: Option<String>,
    #[serde(default)]
    pub medium: Option<String>,
    #[serde(default)]
    pub complex: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct RoutingSection {
    #[serde(default)]
    pub fallback_chain: Vec<String>,
    /// When true and `[routing.thresholds]` is set, replace the first attempt target using the
    /// tier derived from `routing_prompt` (see `build_attempt_chain`).
    #[serde(default)]
    pub auto_route: bool,
    /// Approximate USD budget per SQLite session (optional).
    #[serde(default)]
    pub max_cost_per_session: Option<f64>,
    /// Approximate UTC calendar-month budget per project fingerprint (optional).
    #[serde(default)]
    pub max_cost_per_month: Option<f64>,
    #[serde(default)]
    pub thresholds: Option<RoutingThresholds>,
}

#[derive(Debug, Deserialize, Default)]
pub struct McpClientSection {
    #[serde(default)]
    pub servers: Vec<McpServerEntry>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct McpServerEntry {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Error)]
pub enum ProvidersLoadError {
    #[error("providers.toml not found at {0}; create it (see Cantrik PRD)")]
    NotFound(PathBuf),
    #[error("failed to read providers.toml at {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse providers.toml at {path}: {source}")]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("environment variable {0} is not set (needed for API key)")]
    MissingEnv(String),
    #[error("unknown provider in routing: {0}")]
    UnknownProvider(String),
    #[error(
        "no LLM targets: set [llm] in cantrik.toml and/or [routing].fallback_chain in providers.toml"
    )]
    NoTargets,
}

/// Path to global providers file: `~/.config/cantrik/providers.toml`.
pub fn providers_toml_path() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join(".config")
        .join("cantrik")
        .join("providers.toml")
}

/// Load providers from disk. Fails if file is missing.
pub fn load_providers_toml(path: &Path) -> Result<ProvidersToml, ProvidersLoadError> {
    if !path.exists() {
        return Err(ProvidersLoadError::NotFound(path.to_path_buf()));
    }
    let contents = fs::read_to_string(path).map_err(|source| ProvidersLoadError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let mut file: ProvidersToml =
        toml::from_str(&contents).map_err(|source| ProvidersLoadError::Parse {
            path: path.to_path_buf(),
            source,
        })?;
    // Expand ${VAR} in api keys after parse
    if let Some(ref mut a) = file.providers.anthropic
        && let Some(ref key) = a.api_key
    {
        a.api_key = Some(expand_env_placeholders(key)?);
    }
    if let Some(ref mut g) = file.providers.gemini
        && let Some(ref key) = g.api_key
    {
        g.api_key = Some(expand_env_placeholders(key)?);
    }
    if let Some(ref mut o) = file.providers.openai
        && let Some(ref key) = o.api_key
    {
        o.api_key = Some(expand_env_placeholders(key)?);
    }
    if let Some(ref mut a) = file.providers.azure
        && let Some(ref key) = a.api_key
    {
        a.api_key = Some(expand_env_placeholders(key)?);
    }
    if let Some(ref mut r) = file.providers.openrouter
        && let Some(ref key) = r.api_key
    {
        r.api_key = Some(expand_env_placeholders(key)?);
    }
    if let Some(ref mut g) = file.providers.groq
        && let Some(ref key) = g.api_key
    {
        g.api_key = Some(expand_env_placeholders(key)?);
    }
    Ok(file)
}

/// Replace `${VAR}` with `std::env::var(VAR)`. Fails if unset.
pub fn expand_env_placeholders(value: &str) -> Result<String, ProvidersLoadError> {
    let t = value.trim();
    if let Some(inner) = t.strip_prefix("${").and_then(|s| s.strip_suffix('}')) {
        let var = inner.trim();
        if var.is_empty() {
            return Err(ProvidersLoadError::MissingEnv("(empty)".into()));
        }
        return env::var(var).map_err(|_| ProvidersLoadError::MissingEnv(var.into()));
    }
    Ok(value.to_string())
}

fn anthropic_key(section: Option<&AnthropicSection>) -> Result<String, ProvidersLoadError> {
    if let Some(s) = section
        && let Some(ref k) = s.api_key
        && !k.is_empty()
    {
        return Ok(k.clone());
    }
    env::var("ANTHROPIC_API_KEY")
        .map_err(|_| ProvidersLoadError::MissingEnv("ANTHROPIC_API_KEY".into()))
}

fn gemini_key(section: Option<&GeminiSection>) -> Result<String, ProvidersLoadError> {
    if let Some(s) = section
        && let Some(ref k) = s.api_key
        && !k.is_empty()
    {
        return Ok(k.clone());
    }
    env::var("GEMINI_API_KEY").map_err(|_| ProvidersLoadError::MissingEnv("GEMINI_API_KEY".into()))
}

fn openai_key(section: Option<&OpenAiSection>) -> Result<String, ProvidersLoadError> {
    if let Some(s) = section
        && let Some(ref k) = s.api_key
        && !k.is_empty()
    {
        return Ok(k.clone());
    }
    env::var("OPENAI_API_KEY").map_err(|_| ProvidersLoadError::MissingEnv("OPENAI_API_KEY".into()))
}

fn azure_key(section: Option<&AzureOpenAiSection>) -> Result<String, ProvidersLoadError> {
    if let Some(s) = section
        && let Some(ref k) = s.api_key
        && !k.is_empty()
    {
        return Ok(k.clone());
    }
    env::var("AZURE_OPENAI_API_KEY")
        .map_err(|_| ProvidersLoadError::MissingEnv("AZURE_OPENAI_API_KEY".into()))
}

fn openrouter_key(section: Option<&OpenRouterSection>) -> Result<String, ProvidersLoadError> {
    if let Some(s) = section
        && let Some(ref k) = s.api_key
        && !k.is_empty()
    {
        return Ok(k.clone());
    }
    env::var("OPENROUTER_API_KEY")
        .map_err(|_| ProvidersLoadError::MissingEnv("OPENROUTER_API_KEY".into()))
}

fn groq_key(section: Option<&GroqSection>) -> Result<String, ProvidersLoadError> {
    if let Some(s) = section
        && let Some(ref k) = s.api_key
        && !k.is_empty()
    {
        return Ok(k.clone());
    }
    env::var("GROQ_API_KEY").map_err(|_| ProvidersLoadError::MissingEnv("GROQ_API_KEY".into()))
}

/// API key for provider (never log this).
pub fn resolve_api_key(
    kind: ProviderKind,
    providers: &ProvidersToml,
) -> Result<String, ProvidersLoadError> {
    match kind {
        ProviderKind::Anthropic => anthropic_key(providers.providers.anthropic.as_ref()),
        ProviderKind::Gemini => gemini_key(providers.providers.gemini.as_ref()),
        ProviderKind::Ollama => Ok(String::new()),
        ProviderKind::OpenAi => openai_key(providers.providers.openai.as_ref()),
        ProviderKind::AzureOpenAi => azure_key(providers.providers.azure.as_ref()),
        ProviderKind::OpenRouter => openrouter_key(providers.providers.openrouter.as_ref()),
        ProviderKind::Groq => groq_key(providers.providers.groq.as_ref()),
    }
}

pub fn resolve_default_model(kind: ProviderKind, providers: &ProvidersToml) -> Option<String> {
    match kind {
        ProviderKind::Anthropic => providers
            .providers
            .anthropic
            .as_ref()
            .and_then(|s| s.default_model.clone()),
        ProviderKind::Gemini => providers
            .providers
            .gemini
            .as_ref()
            .and_then(|s| s.default_model.clone()),
        ProviderKind::Ollama => providers
            .providers
            .ollama
            .as_ref()
            .and_then(|s| s.default_model.clone()),
        ProviderKind::OpenAi => providers
            .providers
            .openai
            .as_ref()
            .and_then(|s| s.default_model.clone()),
        ProviderKind::AzureOpenAi => providers
            .providers
            .azure
            .as_ref()
            .and_then(|s| s.default_deployment.clone()),
        ProviderKind::OpenRouter => providers
            .providers
            .openrouter
            .as_ref()
            .and_then(|s| s.default_model.clone()),
        ProviderKind::Groq => providers
            .providers
            .groq
            .as_ref()
            .and_then(|s| s.default_model.clone()),
    }
}

fn default_openai_base() -> String {
    "https://api.openai.com/v1".to_string()
}

fn default_openrouter_base() -> String {
    "https://openrouter.ai/api/v1".to_string()
}

fn default_groq_base() -> String {
    "https://api.groq.com/openai/v1".to_string()
}

/// OpenAI `base_url` including `/v1`.
pub fn openai_api_base(providers: &ProvidersToml) -> String {
    providers
        .providers
        .openai
        .as_ref()
        .and_then(|s| s.base_url.as_ref())
        .map(|u| u.trim_end_matches('/').to_string())
        .unwrap_or_else(default_openai_base)
}

pub fn openrouter_api_base(providers: &ProvidersToml) -> String {
    providers
        .providers
        .openrouter
        .as_ref()
        .and_then(|s| s.base_url.as_ref())
        .map(|u| u.trim_end_matches('/').to_string())
        .unwrap_or_else(default_openrouter_base)
}

pub fn groq_api_base(providers: &ProvidersToml) -> String {
    providers
        .providers
        .groq
        .as_ref()
        .and_then(|s| s.base_url.as_ref())
        .map(|u| u.trim_end_matches('/').to_string())
        .unwrap_or_else(default_groq_base)
}

/// Full URL for Azure chat completions (deployment = model id in routing).
pub fn azure_chat_completions_url(
    providers: &ProvidersToml,
    deployment: &str,
) -> Result<String, ProvidersLoadError> {
    let sec = providers
        .providers
        .azure
        .as_ref()
        .ok_or_else(|| ProvidersLoadError::UnknownProvider("azure section missing".into()))?;
    let api_version = sec.api_version.as_deref().unwrap_or("2024-02-01-preview");
    let ep = sec.endpoint.trim_end_matches('/');
    Ok(format!(
        "{ep}/openai/deployments/{deployment}/chat/completions?api-version={api_version}"
    ))
}

pub fn ollama_base_url(providers: &ProvidersToml) -> String {
    providers
        .providers
        .ollama
        .as_ref()
        .map(|o| o.base_url.trim_end_matches('/').to_string())
        .unwrap_or_else(|| default_ollama_base().trim_end_matches('/').to_string())
}

/// Parse `provider/model` or `provider` (model optional).
pub fn parse_route_entry(
    entry: &str,
) -> Result<(ProviderKind, Option<String>), ProvidersLoadError> {
    let entry = entry.trim();
    if entry.is_empty() {
        return Err(ProvidersLoadError::UnknownProvider("(empty)".into()));
    }
    let (p, m) = match entry.split_once('/') {
        Some((p, m)) => (p.trim(), Some(m.trim().to_string())),
        None => (entry, None),
    };
    let kind =
        ProviderKind::parse(p).ok_or_else(|| ProvidersLoadError::UnknownProvider(p.into()))?;
    let model = m.filter(|s| !s.is_empty());
    Ok((kind, model))
}

/// Resolve a single routing line to a concrete model (defaults from `providers.toml` when omitted).
pub fn route_entry_to_target(
    entry: &str,
    providers: &ProvidersToml,
) -> Result<ProviderTarget, ProvidersLoadError> {
    let (kind, model_opt) = parse_route_entry(entry)?;
    let model = model_opt
        .or_else(|| resolve_default_model(kind, providers))
        .ok_or(ProvidersLoadError::NoTargets)?;
    Ok(ProviderTarget { kind, model })
}

/// Build ordered unique targets: primary from `cantrik.toml` `[llm]`, then `[routing].fallback_chain`.
pub fn build_attempt_chain(
    app_llm_provider: Option<&str>,
    app_llm_model: Option<&str>,
    providers: &ProvidersToml,
) -> Result<Vec<ProviderTarget>, ProvidersLoadError> {
    let mut out: Vec<ProviderTarget> = Vec::new();

    if let Some(p) = app_llm_provider {
        let kind = ProviderKind::parse(p).ok_or_else(|| {
            ProvidersLoadError::UnknownProvider(format!(
                "{p} in cantrik.toml [llm].provider (try: anthropic, gemini, openai, azure, openrouter, groq, ollama)"
            ))
        })?;
        let model = app_llm_model
            .map(str::to_string)
            .filter(|s| !s.is_empty())
            .or_else(|| resolve_default_model(kind, providers))
            .ok_or(ProvidersLoadError::NoTargets)?;
        out.push(ProviderTarget { kind, model });
    }

    if let Some(r) = &providers.routing {
        for line in &r.fallback_chain {
            let (kind, model_opt) = parse_route_entry(line)?;
            let model = model_opt
                .or_else(|| resolve_default_model(kind, providers))
                .ok_or(ProvidersLoadError::NoTargets)?;
            let t = ProviderTarget { kind, model };
            if !out.iter().any(|x| x == &t) {
                out.push(t);
            }
        }
    }

    if out.is_empty() {
        return Err(ProvidersLoadError::NoTargets);
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parse_route() {
        let (k, m) = parse_route_entry("anthropic/claude-3-5-haiku").expect("ok");
        assert_eq!(k, ProviderKind::Anthropic);
        assert_eq!(m.as_deref(), Some("claude-3-5-haiku"));
        let (k2, m2) = parse_route_entry("OLLAMA").expect("ok");
        assert_eq!(k2, ProviderKind::Ollama);
        assert!(m2.is_none());
        let (k3, _) = parse_route_entry("openai/gpt-4o-mini").expect("ok");
        assert_eq!(k3, ProviderKind::OpenAi);
        let (k4, _) = parse_route_entry("groq").expect("ok");
        assert_eq!(k4, ProviderKind::Groq);
    }

    #[test]
    fn build_chain_with_primary_and_fallback() {
        let prov = ProvidersToml {
            providers: ProviderSections {
                anthropic: Some(AnthropicSection {
                    api_key: None,
                    default_model: Some("m-a".into()),
                }),
                gemini: Some(GeminiSection {
                    api_key: None,
                    default_model: Some("m-g".into()),
                }),
                ollama: Some(OllamaSection {
                    base_url: "http://x".into(),
                    default_model: Some("m-o".into()),
                    _embed_model: None,
                }),
                ..Default::default()
            },
            routing: Some(RoutingSection {
                fallback_chain: vec![
                    "gemini/m-g2".into(),
                    "ollama".into(), // uses default m-o
                ],
                ..Default::default()
            }),
            mcp_client: None,
        };
        let chain = build_attempt_chain(Some("anthropic"), None, &prov).expect("chain");
        assert_eq!(
            chain,
            vec![
                ProviderTarget {
                    kind: ProviderKind::Anthropic,
                    model: "m-a".into()
                },
                ProviderTarget {
                    kind: ProviderKind::Gemini,
                    model: "m-g2".into()
                },
                ProviderTarget {
                    kind: ProviderKind::Ollama,
                    model: "m-o".into()
                },
            ]
        );
    }

    #[test]
    fn expand_env_roundtrip() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let var = format!("CANTRIK_TEST_KEY_{unique}");
        // SAFETY: test-only env mutation; single-threaded test.
        unsafe {
            env::set_var(&var, "secret123");
        }
        let v = expand_env_placeholders(&format!("${{{var}}}")).expect("expand");
        assert_eq!(v, "secret123");
        unsafe {
            env::remove_var(&var);
        }
    }

    #[test]
    fn build_chain_fallback_only() {
        let prov = ProvidersToml {
            providers: ProviderSections {
                ollama: Some(OllamaSection {
                    base_url: "http://127.0.0.1:11434".into(),
                    default_model: Some("llama3.2".into()),
                    _embed_model: None,
                }),
                ..Default::default()
            },
            routing: Some(RoutingSection {
                fallback_chain: vec!["ollama".into()],
                ..Default::default()
            }),
            mcp_client: None,
        };
        let chain = build_attempt_chain(None, None, &prov).expect("chain");
        assert_eq!(
            chain,
            vec![ProviderTarget {
                kind: ProviderKind::Ollama,
                model: "llama3.2".into()
            }]
        );
    }

    #[test]
    fn load_sample_toml_from_temp() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = env::temp_dir().join(format!("cantrik-prov-{unique}"));
        fs::create_dir_all(&dir).expect("dir");
        let path = dir.join("providers.toml");
        let mut f = fs::File::create(&path).expect("file");
        write!(
            f,
            r#"
[providers.anthropic]
api_key = "k"
default_model = "claude-test"

[routing]
fallback_chain = ["gemini/gem-flash"]
"#
        )
        .expect("write");
        let loaded = load_providers_toml(&path).expect("load");
        assert!(loaded.providers.anthropic.is_some());
        assert_eq!(loaded.routing.as_ref().unwrap().fallback_chain.len(), 1);
        let _ = fs::remove_dir_all(&dir);
    }
}
