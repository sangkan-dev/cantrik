use reqwest::Client;
use serde_json::Value;
use sha2::{Digest, Sha256};
use url::Url;

use crate::config::AppConfig;
use crate::llm::providers::{load_providers_toml, providers_toml_path};

use super::SearchError;

const DEFAULT_OLLAMA: &str = "http://127.0.0.1:11434";
const EMBED_PATH: &str = "/api/embed";

/// Ollama HTTP base for embeddings: non-empty `[index].ollama_base`, else same as [`resolve_ollama_base`].
pub fn resolve_embed_base_url(config: &AppConfig) -> String {
    match config
        .index
        .ollama_base
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        None => resolve_ollama_base(),
        Some(b) => normalize_base(b),
    }
}

/// `OLLAMA_HOST`, then `providers.toml` `[providers.ollama]`, then `http://127.0.0.1:11434`.
pub fn resolve_ollama_base() -> String {
    if let Ok(h) = std::env::var("OLLAMA_HOST") {
        let t = h.trim();
        if !t.is_empty() {
            return normalize_base(t);
        }
    }
    let path = providers_toml_path();
    if let Ok(prov) = load_providers_toml(&path) {
        return normalize_base(&crate::llm::providers::ollama_base_url(&prov));
    }
    normalize_base(DEFAULT_OLLAMA)
}

pub(super) fn normalize_base(s: &str) -> String {
    let t = s.trim_end_matches('/');
    if t.starts_with("http://") || t.starts_with("https://") {
        t.to_string()
    } else {
        format!("http://{t}")
    }
}

fn embed_url(base: &str) -> Result<Url, SearchError> {
    let base = base.trim_end_matches('/');
    Url::parse(base)
        .and_then(|u| u.join(EMBED_PATH.trim_start_matches('/')))
        .map_err(|e| SearchError::Http(format!("bad Ollama URL: {e}")))
}

/// Single text embedding via Ollama `/api/embed`.
pub async fn embed_one(
    client: &Client,
    base: &str,
    model: &str,
    text: &str,
) -> Result<Vec<f32>, SearchError> {
    let url = embed_url(base)?;
    let body = serde_json::json!({
        "model": model,
        "input": text,
    });
    let response = client
        .post(url)
        .json(&body)
        .send()
        .await
        .map_err(|e| SearchError::Http(e.to_string()))?;
    if !response.status().is_success() {
        let status = response.status();
        let err_body = response
            .text()
            .await
            .unwrap_or_else(|_| String::from("(no body)"));
        return Err(SearchError::Http(format!(
            "ollama embed {status}: {}",
            err_body.chars().take(500).collect::<String>()
        )));
    }
    let v: Value = response
        .json()
        .await
        .map_err(|e| SearchError::Http(e.to_string()))?;
    parse_embed_response(v)
}

/// Batch embeddings (Ollama accepts `input` as array of strings).
pub async fn embed_batch(
    client: &Client,
    base: &str,
    model: &str,
    texts: &[String],
) -> Result<Vec<Vec<f32>>, SearchError> {
    if texts.is_empty() {
        return Ok(vec![]);
    }
    if texts.len() == 1 {
        return Ok(vec![embed_one(client, base, model, &texts[0]).await?]);
    }
    let url = embed_url(base)?;
    let body = serde_json::json!({
        "model": model,
        "input": texts,
    });
    let response = client
        .post(url)
        .json(&body)
        .send()
        .await
        .map_err(|e| SearchError::Http(e.to_string()))?;
    if !response.status().is_success() {
        let status = response.status();
        let err_body = response
            .text()
            .await
            .unwrap_or_else(|_| String::from("(no body)"));
        return Err(SearchError::Http(format!(
            "ollama embed batch {status}: {}",
            err_body.chars().take(500).collect::<String>()
        )));
    }
    let v: Value = response
        .json()
        .await
        .map_err(|e| SearchError::Http(e.to_string()))?;
    parse_embed_batch_response(v)
}

fn parse_embed_response(v: Value) -> Result<Vec<f32>, SearchError> {
    if let Some(arr) = v.get("embedding").and_then(|x| x.as_array()) {
        return json_array_to_f32(arr);
    }
    if let Some(outer) = v.get("embeddings").and_then(|x| x.as_array())
        && let Some(first) = outer.first().and_then(|x| x.as_array())
    {
        return json_array_to_f32(first);
    }
    Err(SearchError::EmbedApi(
        "missing embedding in Ollama response".into(),
    ))
}

fn parse_embed_batch_response(v: Value) -> Result<Vec<Vec<f32>>, SearchError> {
    let Some(outer) = v.get("embeddings").and_then(|x| x.as_array()) else {
        return Err(SearchError::EmbedApi(
            "missing embeddings array in Ollama response".into(),
        ));
    };
    let mut out = Vec::with_capacity(outer.len());
    for item in outer {
        let arr = item
            .as_array()
            .ok_or_else(|| SearchError::EmbedApi("embedding row not array".into()))?;
        out.push(json_array_to_f32(arr)?);
    }
    Ok(out)
}

fn json_array_to_f32(arr: &[Value]) -> Result<Vec<f32>, SearchError> {
    let mut v = Vec::with_capacity(arr.len());
    for x in arr {
        let n = x
            .as_f64()
            .ok_or_else(|| SearchError::EmbedApi("non-numeric in embedding".into()))?;
        v.push(n as f32);
    }
    Ok(v)
}

pub fn chunk_id_for(path: &str, start_byte: usize, end_byte: usize) -> String {
    let raw = format!("{path}\0{start_byte}\0{end_byte}");
    hex::encode(Sha256::digest(raw.as_bytes()))
}

pub fn content_hash(text: &str) -> String {
    hex::encode(Sha256::digest(text.as_bytes()))
}

pub fn embed_input_for_chunk(path: &str, symbol: &str, language: &str, source: &str) -> String {
    let max_body: usize = 12_000;
    let body: String = source.chars().take(max_body).collect();
    format!("{path}\n{symbol}\n{language}\n{body}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_embed_single() {
        let j = serde_json::json!({"embedding": [0.5, 1.0, -0.25]});
        let v = parse_embed_response(j).unwrap();
        assert_eq!(v.len(), 3);
        assert!((v[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn parse_embed_batch() {
        let j = serde_json::json!({"embeddings": [[1.0, 0.0], [0.0, 1.0]]});
        let v = parse_embed_batch_response(j).unwrap();
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn chunk_id_stable() {
        assert_eq!(chunk_id_for("a/b.rs", 1, 10), chunk_id_for("a/b.rs", 1, 10));
        assert_ne!(chunk_id_for("a/b.rs", 1, 10), chunk_id_for("a/b.rs", 1, 11));
    }
}
