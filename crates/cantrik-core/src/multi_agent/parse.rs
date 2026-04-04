use serde::Deserialize;
use thiserror::Error;

use super::SubTask;

#[derive(Debug, Error)]
pub enum MultiAgentParseError {
    #[error("no JSON object in model output")]
    NoJson,
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

fn extract_json_object(raw: &str) -> Result<&str, MultiAgentParseError> {
    let t = raw.trim();
    let unfence = if let Some(rest) = t.strip_prefix("```json") {
        rest.trim_start().split("```").next().unwrap_or(rest).trim()
    } else if let Some(rest) = t.strip_prefix("```") {
        rest.trim_start().split("```").next().unwrap_or(rest).trim()
    } else {
        t
    };
    let start = unfence.find('{').ok_or(MultiAgentParseError::NoJson)?;
    let end = unfence.rfind('}').ok_or(MultiAgentParseError::NoJson)?;
    if end <= start {
        return Err(MultiAgentParseError::NoJson);
    }
    Ok(&unfence[start..=end])
}

#[derive(Debug, Deserialize)]
struct DecomposeWire {
    subtasks: Vec<SubTaskWire>,
}

#[derive(Debug, Deserialize)]
struct SubTaskWire {
    id: String,
    instruction: String,
}

pub fn parse_decompose(raw: &str) -> Result<Vec<SubTask>, MultiAgentParseError> {
    let slice = extract_json_object(raw)?;
    let w: DecomposeWire = serde_json::from_str(slice)?;
    Ok(w.subtasks
        .into_iter()
        .map(|s| SubTask {
            id: s.id,
            instruction: s.instruction,
        })
        .collect())
}

#[derive(Debug, Deserialize)]
struct SummaryWire {
    #[serde(default)]
    summary: String,
    #[serde(default)]
    detail: Option<String>,
}

pub fn parse_subagent_summary(raw: &str, max_chars: usize) -> String {
    let ell = '…'.len_utf8();
    let clamp = |s: &str| {
        if s.len() <= max_chars {
            return s.to_string();
        }
        let mut out = String::new();
        for ch in s.chars() {
            if out.len() + ch.len_utf8() + ell > max_chars {
                break;
            }
            out.push(ch);
        }
        if out.len() < s.len() {
            out.push('…');
        }
        out
    };
    let Ok(slice) = extract_json_object(raw) else {
        return clamp(raw.trim());
    };
    let Ok(w) = serde_json::from_str::<SummaryWire>(slice) else {
        return clamp(raw.trim());
    };
    let mut s = w.summary;
    if s.is_empty() {
        s = w.detail.unwrap_or_default();
    }
    clamp(&s)
}
