use std::path::Path;
use std::process::ExitCode;

use cantrik_core::config::{load_merged_config, resolve_config_paths};
use cantrik_core::llm::{ProviderKind, load_providers_toml, providers_toml_path, resolve_api_key};

pub(crate) fn run(cwd: &Path) -> ExitCode {
    let paths = resolve_config_paths(cwd);
    let providers_path = providers_toml_path();

    println!("doctor: Cantrik {}", env!("CARGO_PKG_VERSION"));
    println!("  global config : {}", paths.global.display());
    println!("  project config: {}", paths.project.display());
    println!("  providers.toml: {}", providers_path.display());

    match load_providers_toml(&providers_path) {
        Ok(prov) => {
            println!("  providers load: OK");
            for kind in [
                ProviderKind::Anthropic,
                ProviderKind::Gemini,
                ProviderKind::Ollama,
            ] {
                let status = match kind {
                    ProviderKind::Ollama => "local (no API key required)",
                    _ => {
                        if resolve_api_key(kind, &prov).is_ok() {
                            "API key / env ready"
                        } else {
                            "missing API key (set in providers.toml or env)"
                        }
                    }
                };
                println!("  - {}: {status}", kind.as_str());
            }
        }
        Err(e) => println!("  providers: {e}"),
    }

    match load_merged_config(cwd) {
        Ok(config) => {
            println!("  config load: OK");
            if let Some(lang) = config.ui.language.as_deref() {
                println!("  ui.language  : {lang}");
            }
            if let Some(p) = config.llm.provider.as_deref() {
                println!("  llm.provider : {p}");
            }
            if let Some(m) = config.llm.model.as_deref() {
                println!("  llm.model    : {m}");
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("  config load: FAILED — {error}");
            ExitCode::from(1)
        }
    }
}
