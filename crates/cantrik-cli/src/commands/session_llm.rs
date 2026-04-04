//! Stream an LLM reply with Tier-2 session persistence (Sprint 7).

use std::path::Path;

use cantrik_core::config::AppConfig;
use cantrik_core::llm::{self, LlmError};
use cantrik_core::session::{
    self, append_message, build_llm_prompt, connect_pool, maybe_summarize_session,
    open_or_create_session,
};

#[derive(Debug)]
pub enum SessionPromptError {
    Session(session::SessionError),
    Llm(LlmError),
}

impl std::fmt::Display for SessionPromptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionPromptError::Session(e) => write!(f, "{e}"),
            SessionPromptError::Llm(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for SessionPromptError {}

impl From<session::SessionError> for SessionPromptError {
    fn from(e: session::SessionError) -> Self {
        Self::Session(e)
    }
}

impl From<LlmError> for SessionPromptError {
    fn from(e: LlmError) -> Self {
        Self::Llm(e)
    }
}

/// Connect DB, ensure session, optionally summarize, append user message, stream LLM, append assistant.
pub async fn stream_with_session(
    cwd: &Path,
    config: &AppConfig,
    user_prompt: &str,
    on_chunk: &mut impl FnMut(&str) -> Result<(), LlmError>,
) -> Result<(), SessionPromptError> {
    let pool = connect_pool().await?;
    let sid = open_or_create_session(&pool, cwd).await?;
    maybe_summarize_session(&pool, &sid, cwd, config).await?;
    append_message(&pool, &sid, "user", user_prompt).await?;

    let full_prompt = build_llm_prompt(&pool, &sid, cwd, config, user_prompt).await?;
    let mut assistant = String::new();
    llm::ask_stream_chunks(config, &full_prompt, &mut |s| {
        assistant.push_str(s);
        on_chunk(s)
    })
    .await?;
    append_message(&pool, &sid, "assistant", &assistant).await?;
    Ok(())
}

/// Same as [`stream_with_session`] but returns the full assistant text (Sprint 10 planning / experiment).
pub async fn complete_with_session(
    cwd: &Path,
    config: &AppConfig,
    user_prompt: &str,
) -> Result<String, SessionPromptError> {
    let mut acc = String::new();
    stream_with_session(cwd, config, user_prompt, &mut |s| {
        acc.push_str(s);
        Ok(())
    })
    .await?;
    Ok(acc)
}
