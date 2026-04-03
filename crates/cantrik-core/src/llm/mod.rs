//! LLM bridge: multi-provider streaming (Sprint 3).

mod anthropic;
mod gemini;
mod ollama;
mod openai_compat;
pub mod providers;
mod stream_util;

use std::time::Duration;

use reqwest::Client;
use thiserror::Error;

pub use providers::{
    ProviderKind, ProviderTarget, ProvidersToml, azure_chat_completions_url, build_attempt_chain,
    groq_api_base, load_providers_toml, ollama_base_url, openai_api_base, openrouter_api_base,
    providers_toml_path, resolve_api_key,
};

use crate::config::AppConfig;
use providers::ProvidersLoadError;

#[derive(Debug, Error)]
pub enum LlmError {
    #[error(transparent)]
    Providers(#[from] ProvidersLoadError),
    #[error("HTTP / transport: {0}")]
    Http(String),
    #[error("stream parse: {0}")]
    StreamParse(String),
    #[error("all configured providers failed; last error: {0}")]
    Exhausted(String),
}

fn http_client() -> Result<Client, LlmError> {
    Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| LlmError::Http(e.to_string()))
}

/// Stream assistant text chunks to `on_chunk`. Tries primary `[llm]` target then `fallback_chain`.
pub async fn ask_stream_chunks(
    app: &AppConfig,
    prompt: &str,
    on_chunk: &mut impl FnMut(&str) -> Result<(), LlmError>,
) -> Result<(), LlmError> {
    let path = providers_toml_path();
    let prov = load_providers_toml(&path)?;
    let chain = build_attempt_chain(app.llm.provider.as_deref(), app.llm.model.as_deref(), &prov)?;
    let client = http_client()?;

    let mut last_err: Option<LlmError> = None;

    for target in chain {
        let api_key = match resolve_api_key(target.kind, &prov) {
            Ok(k) => k,
            Err(e) => {
                last_err = Some(LlmError::Providers(e));
                continue;
            }
        };

        let mut any_emitted = false;
        let mut forward = |s: &str| -> Result<(), LlmError> {
            if !s.is_empty() {
                any_emitted = true;
            }
            on_chunk(s)
        };

        let attempt = match target.kind {
            ProviderKind::Anthropic => {
                anthropic::stream_chat(&client, &api_key, &target.model, prompt, &mut forward).await
            }
            ProviderKind::Gemini => {
                gemini::stream_chat(&client, &api_key, &target.model, prompt, &mut forward).await
            }
            ProviderKind::Ollama => {
                let base = ollama_base_url(&prov);
                ollama::stream_chat(&client, &base, &target.model, prompt, &mut forward).await
            }
            ProviderKind::OpenAi => {
                let base = openai_api_base(&prov);
                let url = format!("{base}/chat/completions");
                openai_compat::stream_chat_completions(
                    &client,
                    &url,
                    &api_key,
                    Some(target.model.as_str()),
                    prompt,
                    &mut forward,
                )
                .await
            }
            ProviderKind::AzureOpenAi => {
                match azure_chat_completions_url(&prov, target.model.as_str()) {
                    Ok(url) => {
                        openai_compat::stream_chat_completions(
                            &client,
                            &url,
                            &api_key,
                            None,
                            prompt,
                            &mut forward,
                        )
                        .await
                    }
                    Err(e) => Err(LlmError::Providers(e)),
                }
            }
            ProviderKind::OpenRouter => {
                let base = openrouter_api_base(&prov);
                let url = format!("{base}/chat/completions");
                openai_compat::stream_chat_completions(
                    &client,
                    &url,
                    &api_key,
                    Some(target.model.as_str()),
                    prompt,
                    &mut forward,
                )
                .await
            }
            ProviderKind::Groq => {
                let base = groq_api_base(&prov);
                let url = format!("{base}/chat/completions");
                openai_compat::stream_chat_completions(
                    &client,
                    &url,
                    &api_key,
                    Some(target.model.as_str()),
                    prompt,
                    &mut forward,
                )
                .await
            }
        };

        match attempt {
            Ok(()) => return Ok(()),
            Err(e) => {
                if any_emitted {
                    return Err(e);
                }
                last_err = Some(e);
            }
        }
    }

    Err(last_err.map_or_else(
        || LlmError::Exhausted("no provider attempts".into()),
        |e| LlmError::Exhausted(e.to_string()),
    ))
}

/// Collect a full assistant string (used e.g. for session summarization).
pub async fn ask_complete_text(app: &AppConfig, prompt: &str) -> Result<String, LlmError> {
    let mut out = String::new();
    ask_stream_chunks(app, prompt, &mut |s| {
        out.push_str(s);
        Ok(())
    })
    .await?;
    Ok(out)
}
