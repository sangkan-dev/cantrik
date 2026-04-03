use reqwest::Client;
use serde_json::Value;

use super::LlmError;
use super::stream_util::for_each_line;

/// Google AI Studio `streamGenerateContent` (REST).
pub async fn stream_chat(
    client: &Client,
    api_key: &str,
    model: &str,
    prompt: &str,
    on_text: &mut impl FnMut(&str) -> Result<(), LlmError>,
) -> Result<(), LlmError> {
    let endpoint = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{model}:streamGenerateContent"
    );
    let mut url = url::Url::parse(&endpoint).map_err(|e| LlmError::Http(e.to_string()))?;
    url.query_pairs_mut().append_pair("key", api_key);
    let body = serde_json::json!({
        "contents": [{
            "parts": [{"text": prompt}]
        }]
    });
    let response = client
        .post(url)
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
            "gemini {status}: {}",
            err_body.chars().take(500).collect::<String>()
        )));
    }
    let stream = response.bytes_stream();
    for_each_line(stream, |line| {
        let line = line.trim();
        let json_str = if let Some(rest) = line.strip_prefix("data:") {
            rest.trim()
        } else {
            line
        };
        if json_str.is_empty() {
            return Ok(());
        }
        let v: Value = serde_json::from_str(json_str)
            .map_err(|e| LlmError::StreamParse(format!("gemini stream json: {e}")))?;
        if let Some(candidates) = v["candidates"].as_array() {
            for c in candidates {
                if let Some(parts) = c["content"]["parts"].as_array() {
                    for p in parts {
                        if let Some(t) = p["text"].as_str()
                            && !t.is_empty()
                        {
                            on_text(t)?;
                        }
                    }
                }
            }
        }
        Ok(())
    })
    .await
}
