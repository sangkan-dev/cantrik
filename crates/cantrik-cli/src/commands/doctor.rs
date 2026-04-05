use std::path::Path;
use std::process::ExitCode;

use cantrik_core::config::{effective_sandbox_level, load_merged_config, resolve_config_paths};
use cantrik_core::llm::{ProviderKind, load_providers_toml, providers_toml_path, resolve_api_key};
use cantrik_core::session::memory_db_path;

/// Lines for `cantrik doctor` and REPL `/doctor` (same content, no secrets).
pub(crate) fn report_lines(cwd: &Path) -> Vec<String> {
    let mut lines = Vec::new();
    let paths = resolve_config_paths(cwd);
    let providers_path = providers_toml_path();

    lines.push(format!("doctor: Cantrik {}", env!("CARGO_PKG_VERSION")));
    lines.push(format!("  global config : {}", paths.global.display()));
    lines.push(format!("  project config: {}", paths.project.display()));
    lines.push(format!("  providers.toml: {}", providers_path.display()));
    lines.push(format!(
        "  session memory DB: {}",
        memory_db_path().display()
    ));

    match load_providers_toml(&providers_path) {
        Ok(prov) => {
            lines.push("  providers load: OK".to_string());
            for kind in ProviderKind::ALL {
                let status = match kind {
                    ProviderKind::Ollama => "local (no API key required)",
                    ProviderKind::AzureOpenAi => match prov.providers.azure.as_ref() {
                        None => "missing [providers.azure]",
                        Some(sec) => {
                            if sec.endpoint.trim().is_empty() {
                                "incomplete (empty endpoint)"
                            } else if resolve_api_key(kind, &prov).is_ok() {
                                "endpoint + API key ready"
                            } else {
                                "missing API key (set in providers.toml or AZURE_OPENAI_API_KEY)"
                            }
                        }
                    },
                    _ => {
                        if resolve_api_key(kind, &prov).is_ok() {
                            "API key / env ready"
                        } else {
                            "missing API key (set in providers.toml or env)"
                        }
                    }
                };
                lines.push(format!("  - {}: {status}", kind.as_str()));
            }
        }
        Err(e) => lines.push(format!("  providers: {e}")),
    }

    match load_merged_config(cwd) {
        Ok(config) => {
            lines.push("  config load: OK".to_string());
            if let Some(lang) = config.ui.language.as_deref() {
                lines.push(format!("  ui.language  : {lang}"));
            }
            if let Some(p) = config.llm.provider.as_deref() {
                lines.push(format!("  llm.provider : {p}"));
            }
            if let Some(m) = config.llm.model.as_deref() {
                lines.push(format!("  llm.model    : {m}"));
            }
            if cantrik_core::config::effective_llm_offline(&config.llm) {
                lines.push("  llm.offline (effective): true (Ollama loopback only)".to_string());
                lines.push(
                    "  outbound HTTP (fetch/web --approve): blocked in offline mode".to_string(),
                );
            }
            if cantrik_core::config::effective_adaptive_begawan(&config.memory) {
                lines
                    .push("  memory.adaptive_begawan: on (approval memory in prompts)".to_string());
            }
            if let Some(m) = config.index.vector_model.as_deref() {
                lines.push(format!("  index.vector_model: {m}"));
            }
            if let Some(b) = config.index.ollama_base.as_deref()
                && !b.trim().is_empty()
            {
                lines.push(format!("  index.ollama_base : {b}"));
            }
            let sb = effective_sandbox_level(&config.sandbox);
            lines.push(format!("  sandbox.level (effective): {sb:?}"));
            if std::env::var_os("CANTRIK_SANDBOX").as_deref() == Some(std::ffi::OsStr::new("0")) {
                lines.push("  CANTRIK_SANDBOX=0 (bubblewrap disabled — dev only)".to_string());
            }
            if !config.guardrails.forbidden.is_empty() {
                lines.push(format!(
                    "  guardrails.forbidden: {:?}",
                    config.guardrails.forbidden
                ));
            }
        }
        Err(error) => lines.push(format!("  config load: FAILED — {error}")),
    }

    match tokio::runtime::Handle::try_current() {
        Ok(handle) => {
            let line = match tokio::task::block_in_place(|| {
                handle.block_on(cantrik_core::search::table_row_count(cwd))
            }) {
                Ok(None) => "  lance vector index: none (under .cantrik/index/lance/)".to_string(),
                Ok(Some(n)) => {
                    format!("  lance vector index: {n} rows (table `code_chunks`)")
                }
                Err(e) => format!("  lance vector index: error — {e}"),
            };
            lines.push(line);
        }
        Err(_) => lines.push("  lance vector index: no tokio runtime".to_string()),
    }

    lines
}

pub(crate) fn run(cwd: &Path) -> ExitCode {
    let lines = report_lines(cwd);
    let mut config_failed = false;
    for line in &lines {
        if line.contains("config load: FAILED") {
            config_failed = true;
            eprintln!("{line}");
        } else {
            println!("{line}");
        }
    }
    if config_failed {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
