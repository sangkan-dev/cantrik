//! Voice: optional Ollama transcription + local TTS hook (Sprint 18, PRD §4.26).

use reqwest::multipart;
use serde_json::Value;
use thiserror::Error;

use crate::config::AppConfig;
use crate::search::resolve_embed_base_url;

use crate::config::effective_transcription_model;

#[derive(Debug, Error)]
pub enum VoiceError {
    #[error("HTTP: {0}")]
    Http(String),
    #[error("transcription: {0}")]
    Transcription(String),
}

/// POST audio to `{ollama}/api/transcribe` when the server supports it (Ollama builds with transcription).
pub async fn transcribe_ollama(
    config: &AppConfig,
    audio: Vec<u8>,
    filename: &str,
) -> Result<String, VoiceError> {
    let base = resolve_embed_base_url(config);
    let model = effective_transcription_model(&config.ui);
    let url = format!("{}/api/transcribe", base.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| VoiceError::Http(e.to_string()))?;
    let part = multipart::Part::bytes(audio)
        .file_name(filename.to_string())
        .mime_str("application/octet-stream")
        .map_err(|e| VoiceError::Http(e.to_string()))?;
    let form = multipart::Form::new()
        .text("model", model)
        .part("file", part);
    let res = client
        .post(&url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| VoiceError::Http(e.to_string()))?;
    let status = res.status();
    let body = res.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(VoiceError::Transcription(format!(
            "{} — {} (ensure Ollama supports /api/transcribe and a whisper model is pulled; or use cantrik listen --raw-text)",
            status,
            body.chars().take(180).collect::<String>()
        )));
    }
    if let Ok(v) = serde_json::from_str::<Value>(&body) {
        let text = v
            .get("text")
            .and_then(|x| x.as_str())
            .or_else(|| v.get("response").and_then(|x| x.as_str()))
            .unwrap_or("")
            .trim();
        if !text.is_empty() {
            return Ok(text.to_string());
        }
    }
    let plain = body.trim();
    if plain.is_empty() {
        return Err(VoiceError::Transcription(
            "empty transcription in response".into(),
        ));
    }
    Ok(plain.to_string())
}

/// Best-effort local TTS (`espeak` on Linux, `say` on macOS). No-op if disabled or unsupported OS.
pub fn speak_notification(voice_enabled: bool, message: &str) {
    if !voice_enabled || message.is_empty() {
        return;
    }
    let t: String = message
        .chars()
        .filter(|c| !c.is_control())
        .take(220)
        .collect();
    if t.is_empty() {
        return;
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("espeak").arg(&t).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("say").arg(&t).spawn();
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = t;
    }
}
