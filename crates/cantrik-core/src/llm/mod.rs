//! LLM bridge: multi-provider streaming (Sprint 3).

mod anthropic;
pub mod complexity;
mod cost;
mod gemini;
mod ollama;
mod openai_compat;
pub mod providers;
mod stream_util;

pub use complexity::{TaskTier, classify_prompt, resolve_routed_target};

use std::time::Duration;

use reqwest::Client;
use sqlx::SqlitePool;
use thiserror::Error;

pub use providers::{
    ProviderKind, ProviderTarget, ProvidersToml, apply_offline_policy, azure_chat_completions_url,
    build_attempt_chain, groq_api_base, load_providers_toml, ollama_base_is_loopback,
    ollama_base_url, openai_api_base, openrouter_api_base, providers_toml_path, resolve_api_key,
};

use crate::config::{AppConfig, effective_llm_offline};
use providers::ProvidersLoadError;

#[derive(Debug, Error)]
pub enum LlmError {
    #[error(transparent)]
    Providers(#[from] ProvidersLoadError),
    #[error(transparent)]
    UsageLedger(#[from] crate::usage_ledger::UsageLedgerError),
    #[error("HTTP / transport: {0}")]
    Http(String),
    #[error("stream parse: {0}")]
    StreamParse(String),
    #[error("all configured providers failed; last error: {0}")]
    Exhausted(String),
    #[error("LLM budget limit reached: {0}")]
    BudgetExceeded(String),
}

/// Optional SQLite context to enforce caps and record approximate spend (see `providers.toml` `[routing]`).
pub struct LlmUsageContext<'a> {
    pub pool: &'a SqlitePool,
    pub session_id: Option<&'a str>,
    pub project_fingerprint: &'a str,
}

fn apply_auto_route(
    chain: &mut [ProviderTarget],
    providers: &ProvidersToml,
    user_facing_prompt: &str,
) -> Result<(), ProvidersLoadError> {
    let Some(r) = providers.routing.as_ref() else {
        return Ok(());
    };
    if !r.auto_route {
        return Ok(());
    }
    let Some(th) = r.thresholds.as_ref() else {
        return Ok(());
    };
    let tier = classify_prompt(user_facing_prompt);
    match resolve_routed_target(tier, th, providers)? {
        Some(tgt) if !chain.is_empty() => chain[0] = tgt,
        _ => {}
    }
    Ok(())
}

fn tier_label_for_ledger(
    providers: &ProvidersToml,
    routing_prompt: Option<&str>,
) -> Option<&'static str> {
    let r = providers.routing.as_ref()?;
    if !(r.auto_route && r.thresholds.is_some()) {
        return None;
    }
    let p = routing_prompt?;
    Some(classify_prompt(p).as_str())
}

async fn enforce_budget_if_needed(
    prov: &ProvidersToml,
    usage: Option<&LlmUsageContext<'_>>,
) -> Result<(), LlmError> {
    let Some(ctx) = usage else {
        return Ok(());
    };
    let Some(r) = prov.routing.as_ref() else {
        return Ok(());
    };
    let ym = chrono::Utc::now().format("%Y-%m").to_string();
    if let Some(max) = r.max_cost_per_month {
        let spent =
            crate::usage_ledger::month_spend_usd(ctx.pool, ctx.project_fingerprint, &ym).await?;
        if spent >= max {
            return Err(LlmError::BudgetExceeded(format!(
                "monthly approximate spend {spent:.6} USD >= cap {max} (UTC {ym})"
            )));
        }
    }
    if let Some(max) = r.max_cost_per_session {
        let Some(sid) = ctx.session_id else {
            return Ok(());
        };
        let spent =
            crate::usage_ledger::session_spend_usd(ctx.pool, ctx.project_fingerprint, sid).await?;
        if spent >= max {
            return Err(LlmError::BudgetExceeded(format!(
                "session approximate spend {spent:.6} USD >= cap {max}"
            )));
        }
    }
    Ok(())
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
    ask_stream_chunks_with(app, prompt, None, None, on_chunk).await
}

/// Like [`ask_stream_chunks`], with optional user-facing text for smart routing and ledger hooks.
pub async fn ask_stream_chunks_with(
    app: &AppConfig,
    prompt: &str,
    routing_prompt: Option<&str>,
    usage: Option<LlmUsageContext<'_>>,
    on_chunk: &mut impl FnMut(&str) -> Result<(), LlmError>,
) -> Result<(), LlmError> {
    let path = providers_toml_path();
    let prov = load_providers_toml(&path)?;
    enforce_budget_if_needed(&prov, usage.as_ref()).await?;

    let mut chain =
        build_attempt_chain(app.llm.provider.as_deref(), app.llm.model.as_deref(), &prov)?;
    if let Some(rp) = routing_prompt {
        apply_auto_route(&mut chain, &prov, rp).map_err(LlmError::Providers)?;
    }
    if effective_llm_offline(&app.llm) {
        chain = apply_offline_policy(chain, &prov)?;
    }

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
        let mut assistant = String::new();
        let mut forward = |s: &str| -> Result<(), LlmError> {
            if !s.is_empty() {
                any_emitted = true;
                assistant.push_str(s);
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
            Ok(()) => {
                if let Some(ctx) = usage {
                    let cost = cost::approx_cost_usd(
                        target.kind,
                        &target.model,
                        prompt.len(),
                        assistant.len(),
                    );
                    let tier = tier_label_for_ledger(&prov, routing_prompt);
                    crate::usage_ledger::insert_llm_usage(
                        ctx.pool,
                        ctx.session_id,
                        ctx.project_fingerprint,
                        target.kind.as_str(),
                        &target.model,
                        tier,
                        prompt.len() as i64,
                        assistant.len() as i64,
                        cost,
                    )
                    .await?;
                }
                return Ok(());
            }
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
    ask_complete_text_with(app, prompt, None, None).await
}

/// Full-string variant with routing/ledger hooks.
pub async fn ask_complete_text_with(
    app: &AppConfig,
    prompt: &str,
    routing_prompt: Option<&str>,
    usage: Option<LlmUsageContext<'_>>,
) -> Result<String, LlmError> {
    let mut out = String::new();
    ask_stream_chunks_with(app, prompt, routing_prompt, usage, &mut |s| {
        out.push_str(s);
        Ok(())
    })
    .await?;
    Ok(out)
}

#[cfg(test)]
mod routing_tests {
    use super::providers::{
        AnthropicSection, OllamaSection, ProviderKind, ProviderSections, ProvidersToml,
        RoutingSection, RoutingThresholds, build_attempt_chain,
    };
    use super::{apply_auto_route, classify_prompt};

    fn sample_providers() -> ProvidersToml {
        ProvidersToml {
            providers: ProviderSections {
                anthropic: Some(AnthropicSection {
                    api_key: None,
                    default_model: Some("claude-big".into()),
                }),
                ollama: Some(OllamaSection {
                    base_url: "http://127.0.0.1:11434".into(),
                    default_model: Some("llama-local".into()),
                    _embed_model: None,
                }),
                ..Default::default()
            },
            routing: Some(RoutingSection {
                fallback_chain: vec![],
                auto_route: true,
                max_cost_per_session: None,
                max_cost_per_month: None,
                thresholds: Some(RoutingThresholds {
                    simple: Some("anthropic/claude-big".into()),
                    medium: Some("anthropic/claude-big".into()),
                    complex: Some("ollama".into()),
                }),
            }),
            mcp_client: None,
        }
    }

    #[test]
    fn classify_audit_is_complex() {
        assert_eq!(
            classify_prompt("audit the security of this module"),
            super::TaskTier::Complex
        );
    }

    #[test]
    fn auto_route_swaps_primary_for_complex_prompt() {
        let prov = sample_providers();
        let mut chain =
            build_attempt_chain(Some("anthropic"), Some("claude-big"), &prov).expect("c");
        assert_eq!(chain[0].model, "claude-big");
        apply_auto_route(&mut chain, &prov, "full security audit of payment flow").expect("ar");
        assert_eq!(chain[0].kind, ProviderKind::Ollama);
        assert_eq!(chain[0].model, "llama-local");
    }

    #[test]
    fn auto_route_keeps_primary_for_simple_prompt() {
        let prov = sample_providers();
        let mut chain =
            build_attempt_chain(Some("anthropic"), Some("claude-big"), &prov).expect("c");
        apply_auto_route(&mut chain, &prov, "ok").expect("ar");
        assert_eq!(chain[0].kind, ProviderKind::Anthropic);
        assert_eq!(chain[0].model, "claude-big");
    }
}
