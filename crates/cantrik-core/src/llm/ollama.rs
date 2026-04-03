use reqwest::Client;

use super::LlmError;
use super::stream_util::for_each_line;

pub async fn stream_chat(
    client: &Client,
    base_trimmed: &str,
    model: &str,
    prompt: &str,
    on_text: &mut impl FnMut(&str) -> Result<(), LlmError>,
) -> Result<(), LlmError> {
    let url = format!("{base_trimmed}/api/chat");
    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "stream": true
    });
    let response = client
        .post(url)
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
            "ollama {status}: {}",
            err_body.chars().take(500).collect::<String>()
        )));
    }
    let stream = response.bytes_stream();
    for_each_line(stream, |line| {
        let v: serde_json::Value = serde_json::from_str(line)
            .map_err(|e| LlmError::StreamParse(format!("ollama json: {e}")))?;
        if let Some(piece) = v["message"]["content"].as_str()
            && !piece.is_empty()
        {
            on_text(piece)?;
        }
        Ok(())
    })
    .await
}
