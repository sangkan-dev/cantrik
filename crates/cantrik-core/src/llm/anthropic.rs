use reqwest::Client;
use serde_json::Value;

use super::LlmError;
use super::stream_util::for_each_line;

pub async fn stream_chat(
    client: &Client,
    api_key: &str,
    model: &str,
    prompt: &str,
    on_text: &mut impl FnMut(&str) -> Result<(), LlmError>,
) -> Result<(), LlmError> {
    let body = serde_json::json!({
        "model": model,
        "max_tokens": 4096,
        "messages": [{"role": "user", "content": prompt}],
        "stream": true
    });
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| LlmError::Http(e.to_string()))?;
    if !response.status().is_success() {
        let status = response.status();
        let err_body = response
            .text()
            .await
            .unwrap_or_else(|_| String::from("(no body)"));
        return Err(LlmError::Http(format!(
            "anthropic {status}: {}",
            err_body.chars().take(500).collect::<String>()
        )));
    }
    let stream = response.bytes_stream();
    for_each_line(stream, |raw| {
        let line = raw.trim();
        let payload = if let Some(rest) = line.strip_prefix("data:") {
            rest.trim()
        } else if line.starts_with("event:") || line.is_empty() {
            return Ok(());
        } else {
            // Some proxies send bare JSON lines
            line
        };
        if payload.is_empty() || payload == "[DONE]" {
            return Ok(());
        }
        let v: Value = serde_json::from_str(payload)
            .map_err(|e| LlmError::StreamParse(format!("anthropic sse json: {e}")))?;
        if v["type"].as_str() == Some("content_block_delta")
            && v["delta"]["type"].as_str() == Some("text_delta")
            && let Some(t) = v["delta"]["text"].as_str()
            && !t.is_empty()
        {
            on_text(t)?;
        }
        Ok(())
    })
    .await
}
