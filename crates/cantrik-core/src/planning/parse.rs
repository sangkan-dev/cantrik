use serde::Deserialize;

use super::model::Plan;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlanningParseError {
    #[error("no JSON object found in model output")]
    NoJson,
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

/// Strip optional ```json ... ``` fence and find first `{` ... `}` slice (best-effort).
pub fn extract_json_object(raw: &str) -> Result<&str, PlanningParseError> {
    let t = raw.trim();
    let unfence = if let Some(rest) = t.strip_prefix("```json") {
        rest.trim_start().split("```").next().unwrap_or(rest).trim()
    } else if let Some(rest) = t.strip_prefix("```") {
        rest.trim_start().split("```").next().unwrap_or(rest).trim()
    } else {
        t
    };

    let start = unfence.find('{').ok_or(PlanningParseError::NoJson)?;
    let end = unfence.rfind('}').ok_or(PlanningParseError::NoJson)?;
    if end <= start {
        return Err(PlanningParseError::NoJson);
    }
    Ok(&unfence[start..=end])
}

#[derive(Debug, Deserialize)]
struct PlanWire {
    steps: Vec<PlanStepWire>,
}

#[derive(Debug, Deserialize)]
struct PlanStepWire {
    id: String,
    description: String,
    #[serde(default)]
    suggested_action: Option<String>,
}

/// Parse LLM output into a [`Plan`]. On failure, returns a single-step fallback (caller may retry).
pub fn parse_plan_document(raw: &str) -> Result<Plan, PlanningParseError> {
    let slice = extract_json_object(raw)?;
    let w: PlanWire = serde_json::from_str(slice)?;
    if w.steps.is_empty() {
        return Err(PlanningParseError::NoJson);
    }
    Ok(Plan {
        steps: w
            .steps
            .into_iter()
            .map(|s| super::model::PlanStep {
                id: s.id,
                description: s.description,
                suggested_action: s.suggested_action,
            })
            .collect(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepEval {
    pub success: bool,
    pub notes: String,
}

#[derive(Debug, Deserialize)]
struct EvalWire {
    success: bool,
    #[serde(default)]
    notes: String,
}

pub fn parse_eval_response(raw: &str) -> Result<StepEval, PlanningParseError> {
    let slice = extract_json_object(raw)?;
    let w: EvalWire = serde_json::from_str(slice)?;
    Ok(StepEval {
        success: w.success,
        notes: w.notes,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExperimentWrites {
    pub writes: Vec<ExperimentWrite>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExperimentWrite {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
struct ExperimentWire {
    #[serde(default)]
    writes: Vec<WriteEntryWire>,
}

#[derive(Debug, Deserialize)]
struct WriteEntryWire {
    path: String,
    content: String,
}

/// Optional `writes` array from experiment LLM output; empty if missing or invalid.
pub fn parse_experiment_writes(raw: &str) -> ExperimentWrites {
    let Ok(slice) = extract_json_object(raw) else {
        return ExperimentWrites { writes: vec![] };
    };
    let Ok(w) = serde_json::from_str::<ExperimentWire>(slice) else {
        return ExperimentWrites { writes: vec![] };
    };
    ExperimentWrites {
        writes: w
            .writes
            .into_iter()
            .map(|e| ExperimentWrite {
                path: e.path,
                content: e.content,
            })
            .collect(),
    }
}

/// Byte index of the closing `}` for a JSON object starting at `start`, respecting strings and escapes.
fn json_object_end(bytes: &[u8], start: usize) -> Option<usize> {
    if start >= bytes.len() || bytes[start] != b'{' {
        return None;
    }
    let mut depth = 0i32;
    let mut in_str = false;
    let mut escape = false;
    let mut i = start;
    while i < bytes.len() {
        let b = bytes[i];
        if in_str {
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        match b {
            b'"' => in_str = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Collect `writes` from every top-level JSON object in `raw` that deserializes as experiment output.
/// Later objects win on duplicate `path`. Falls back to [`parse_experiment_writes`] if none match.
/// Use this for streamed LLM text that may contain multiple `{"writes":[...]}` blobs.
pub fn parse_experiment_writes_greedy(raw: &str) -> ExperimentWrites {
    let bytes = raw.as_bytes();
    let mut merged: Vec<ExperimentWrite> = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{'
            && let Some(end) = json_object_end(bytes, i)
        {
            let slice = &raw[i..=end];
            if let Ok(w) = serde_json::from_str::<ExperimentWire>(slice) {
                for e in w.writes {
                    merged.retain(|x| x.path != e.path);
                    merged.push(ExperimentWrite {
                        path: e.path,
                        content: e.content,
                    });
                }
            }
            i = end + 1;
            continue;
        }
        i += 1;
    }
    if merged.is_empty() {
        parse_experiment_writes(raw)
    } else {
        ExperimentWrites { writes: merged }
    }
}

/// When [`parse_experiment_writes_greedy`] produced no writes, explains why the model output
/// still looked like a `writes` payload (invalid JSON, truncation, empty array, etc.).
///
/// Call only if `parse_experiment_writes_greedy(raw).writes` is already empty, to avoid an extra scan.
pub fn parse_experiment_writes_failure_hint_after_empty(raw: &str) -> Option<String> {
    if !raw.contains("\"writes\"") || !raw.contains('{') {
        return None;
    }
    let slice = match extract_json_object(raw) {
        Ok(s) => s,
        Err(_) => {
            return Some(
                "Tidak ada objek JSON `{ ... }` utuh — sering karena balasan terpotong atau teks non-JSON \
                 dicampur sehingga `}` penutup tidak cocok."
                    .to_string(),
            );
        }
    };
    match serde_json::from_str::<ExperimentWire>(slice) {
        Ok(w) if w.writes.is_empty() => Some(
            "JSON valid, tetapi array `writes` kosong — tidak ada file yang diusulkan.".to_string(),
        ),
        Ok(_) => None,
        Err(e) => Some(format!(
            "JSON tidak valid di sekitar `writes` (periksa kutip dan escape `\\n` untuk baris baru di dalam `content`): {e}"
        )),
    }
}

#[cfg(test)]
mod greedy_tests {
    use super::*;

    #[test]
    fn failure_hint_truncated_object() {
        let raw = r#"Sure. {"writes":[{"path":"a.py","content":"line1
broken newline not escaped"}"#;
        assert!(parse_experiment_writes_greedy(raw).writes.is_empty());
        let hint = parse_experiment_writes_failure_hint_after_empty(raw);
        assert!(hint.is_some(), "{hint:?}");
    }

    #[test]
    fn failure_hint_empty_writes_array() {
        let raw = r#"{"writes":[],"rationale":"none"}"#;
        assert!(parse_experiment_writes_greedy(raw).writes.is_empty());
        let hint = parse_experiment_writes_failure_hint_after_empty(raw).unwrap();
        assert!(hint.contains("kosong"), "{hint}");
    }

    #[test]
    fn failure_hint_none_when_no_writes_keyword() {
        let raw = "just chatting";
        assert!(parse_experiment_writes_failure_hint_after_empty(raw).is_none());
    }

    #[test]
    fn greedy_merges_two_objects_last_path_wins() {
        let raw = r#"{"writes":[{"path":"a.txt","content":"one"}]} xxx {"writes":[{"path":"a.txt","content":"two"},{"path":"b.txt","content":"y"}]}"#;
        let w = parse_experiment_writes_greedy(raw);
        assert_eq!(w.writes.len(), 2);
        let a = w.writes.iter().find(|x| x.path == "a.txt").unwrap();
        assert_eq!(a.content, "two");
    }
}
