//! Assemble prompt with anchors + summary + tail; optional LLM summarization.

use std::path::Path;

use sqlx::SqlitePool;

use crate::config::{AppConfig, effective_cultural_wisdom};
use crate::cultural_wisdom;

use super::{
    ENV_NO_SUMMARY, MessageEntry, SessionError, latest_summary, list_all_messages_ordered,
    save_summary,
};
use crate::llm;

use super::list_messages_after;
use super::load_anchors_combined;
use crate::skills::{
    format_rules_block, format_skills_block, load_rules_text, select_skills_for_prompt,
};

const DEFAULT_TAIL_MESSAGES: usize = 12;
const DEFAULT_SUMMARIZE_THRESHOLD: u64 = 48_000;
const DEFAULT_MAX_CONTEXT_CHARS: u64 = 200_000;

fn summarize_threshold(app: &AppConfig) -> u64 {
    app.memory
        .summarize_threshold_chars
        .unwrap_or(DEFAULT_SUMMARIZE_THRESHOLD)
}

fn max_context_chars(app: &AppConfig) -> u64 {
    app.memory
        .max_context_chars
        .unwrap_or(DEFAULT_MAX_CONTEXT_CHARS)
}

fn tail_messages(app: &AppConfig) -> usize {
    app.memory
        .context_tail_messages
        .map(|n| n as usize)
        .unwrap_or(DEFAULT_TAIL_MESSAGES)
}

fn messages_char_count(msgs: &[MessageEntry]) -> u64 {
    msgs.iter().map(|m| m.content.len() as u64).sum()
}

/// If conversation exceeds threshold, summarize all but the last `tail` messages and store summary.
pub async fn maybe_summarize_session(
    pool: &SqlitePool,
    session_id: &str,
    _cwd: &Path,
    app: &AppConfig,
) -> Result<(), SessionError> {
    if std::env::var(ENV_NO_SUMMARY).is_ok() {
        return Ok(());
    }

    let msgs = list_all_messages_ordered(pool, session_id).await?;
    if msgs.len() <= tail_messages(app) {
        return Ok(());
    }

    let tail_n = tail_messages(app);
    let split_at = msgs.len().saturating_sub(tail_n);
    if split_at == 0 {
        return Ok(());
    }

    let prefix = &msgs[..split_at];
    if messages_char_count(prefix) < summarize_threshold(app) {
        return Ok(());
    }

    let covers_ord = prefix.last().map(|m| m.ordinal).unwrap_or(0);

    let prev = latest_summary(pool, session_id).await?;
    let mut block = String::new();
    if let Some(p) = &prev {
        block.push_str("Previous session summary:\n");
        block.push_str(&p.text);
        block.push_str("\n\n");
    }
    block.push_str("Older conversation to condense (roles preserved):\n");
    for m in prefix {
        block.push_str(&format!("{}: {}\n", m.role, m.content));
    }

    let prompt = format!(
        "Summarize the following conversation for long-term session memory. Keep decisions, names, paths, and constraints. Target roughly 500 tokens or less. Output plain text only, no preamble.\n\n{}",
        block
    );

    let fp = super::project_fingerprint(_cwd);
    let usage = llm::LlmUsageContext {
        pool,
        session_id: Some(session_id),
        project_fingerprint: &fp,
    };
    let summary_text = llm::ask_complete_text_with(app, &prompt, None, Some(usage))
        .await
        .map_err(|e| SessionError::Llm(e.to_string()))?;

    save_summary(pool, session_id, summary_text.trim(), covers_ord).await?;
    Ok(())
}

/// Build full prompt: anchors, summary, retained messages, then current user line.
pub async fn build_llm_prompt(
    pool: &SqlitePool,
    session_id: &str,
    cwd: &Path,
    app: &AppConfig,
    current_user_line: &str,
) -> Result<String, SessionError> {
    let anchors = load_anchors_combined(cwd);
    let sum = latest_summary(pool, session_id).await?;
    let after = sum.as_ref().map(|s| s.covers_up_to_ordinal).unwrap_or(0);
    let mut retained = list_messages_after(pool, session_id, after).await?;

    let mut parts = vec![];

    if !anchors.is_empty() {
        parts.push(format!(
            "The following memory anchors must be respected (do not contradict):\n\n{anchors}\n"
        ));
    }

    if let Some(rules) = load_rules_text(cwd) {
        parts.push(format_rules_block(&rules));
    }

    let skill_chunks = select_skills_for_prompt(cwd, app, current_user_line);
    let skills_block = format_skills_block(&skill_chunks);
    if !skills_block.is_empty() {
        parts.push(skills_block);
    }

    if let Some(block) = cultural_wisdom::prompt_addon(effective_cultural_wisdom(&app.ui)) {
        parts.push(block);
    }

    if let Some(s) = &sum
        && !s.text.is_empty()
    {
        parts.push(format!(
            "Earlier conversation summary (ordinals <= {}):\n{}\n",
            s.covers_up_to_ordinal, s.text
        ));
    }

    // Drop the last user message if it equals current_user_line (we append it at end explicitly)
    if let Some(last) = retained.last()
        && last.role == "user"
        && last.content == current_user_line
    {
        retained.pop();
    }

    if !retained.is_empty() {
        let mut hist = String::from("Recent conversation:\n");
        for m in &retained {
            hist.push_str(&format!("{}: {}\n", m.role, m.content));
        }
        parts.push(hist);
    }

    let mut body = parts.join("\n");
    body.push_str(&format!("\nuser: {current_user_line}\n"));

    // Hard cap
    let max = max_context_chars(app) as usize;
    if body.len() > max {
        body = body[body.len().saturating_sub(max)..].to_string();
        body.insert_str(0, "...[context truncated]\n");
    }

    Ok(body)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::config::{AppConfig, CulturalWisdomLevel};
    use crate::session::{connect_pool, open_or_create_session};
    use crate::skills::ENV_NO_RULES;

    #[tokio::test]
    async fn prompt_includes_project_rules() {
        let dir = std::env::temp_dir().join(format!("cantrik-rules-prompt-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".cantrik")).unwrap();
        fs::write(dir.join(".cantrik/rules.md"), "RULE_LINE_UNIQUE").unwrap();

        unsafe { std::env::remove_var(ENV_NO_RULES) };
        unsafe { std::env::set_var(crate::session::ENV_MEMORY_DB, dir.join("db.sqlite")) };

        let pool = connect_pool().await.expect("pool");
        let sid = open_or_create_session(&pool, &dir).await.expect("session");
        let app = AppConfig::default();
        let body = build_llm_prompt(&pool, &sid, &dir, &app, "hello")
            .await
            .expect("prompt");

        assert!(
            body.contains("RULE_LINE_UNIQUE"),
            "body should contain rules: {body}"
        );
        assert!(body.contains("Project rules"));

        unsafe {
            std::env::remove_var(crate::session::ENV_MEMORY_DB);
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn prompt_includes_cultural_wisdom_when_enabled() {
        let dir = std::env::temp_dir().join(format!("cantrik-cw-prompt-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".cantrik")).unwrap();

        unsafe { std::env::set_var(crate::session::ENV_MEMORY_DB, dir.join("db.sqlite")) };

        let pool = connect_pool().await.expect("pool");
        let sid = open_or_create_session(&pool, &dir).await.expect("session");
        let mut app = AppConfig::default();
        app.ui.cultural_wisdom = Some(CulturalWisdomLevel::Light);
        let body = build_llm_prompt(&pool, &sid, &dir, &app, "hello")
            .await
            .expect("prompt");

        assert!(
            body.contains("cultural wisdom"),
            "expected cultural block in {body}"
        );

        unsafe {
            std::env::remove_var(crate::session::ENV_MEMORY_DB);
        }
        let _ = fs::remove_dir_all(&dir);
    }
}
