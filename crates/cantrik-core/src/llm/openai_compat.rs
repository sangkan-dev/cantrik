//! OpenAI-compatible `POST .../chat/completions` with `stream: true` (SSE).

use reqwest::Client;
use serde_json::Value;

use super::LlmError;
use super::stream_util::for_each_line;

/// `model` is omitted from the JSON body when `None` (e.g. Azure uses deployment in the URL only).
pub async fn stream_chat_completions(
    client: &Client,
    post_url: &str,
    bearer_token: &str,
    model: Option<&str>,
    prompt: &str,
    on_text: &mut impl FnMut(&str) -> Result<(), LlmError>,
) -> Result<(), LlmError> {
    let mut body = serde_json::json!({
        "messages": [{"role": "user", "content": prompt}],
        "stream": true,
    });
    if let Some(m) = model {
        body.as_object_mut()
            .ok_or_else(|| LlmError::StreamParse("chat body".into()))?
            .insert("model".into(), Value::String(m.to_string()));
    }

    let response = client
        .post(post_url)
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {bearer_token}"),
        )
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
            "openai-compatible {status}: {}",
            err_body.chars().take(500).collect::<String>()
        )));
    }

    let stream = response.bytes_stream();
    for_each_line(stream, |raw| {
        let line = raw.trim();
        let payload = if let Some(rest) = line.strip_prefix("data:") {
            rest.trim()
        } else if line.is_empty() || line.starts_with(':') {
            return Ok(());
        } else {
            line
        };
        if payload.is_empty() || payload == "[DONE]" {
            return Ok(());
        }
        let v: Value = serde_json::from_str(payload)
            .map_err(|e| LlmError::StreamParse(format!("openai sse json: {e}")))?;
        if let Some(choices) = v["choices"].as_array() {
            for c in choices {
                if let Some(content) = c["delta"]["content"].as_str()
                    && !content.is_empty()
                {
                    on_text(content)?;
                }
            }
        }
        Ok(())
    })
    .await
}
