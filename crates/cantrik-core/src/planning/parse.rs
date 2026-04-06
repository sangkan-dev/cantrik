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

#[cfg(test)]
mod greedy_tests {
    use super::*;

    #[test]
    fn greedy_merges_two_objects_last_path_wins() {
        let raw = r#"{"writes":[{"path":"a.txt","content":"one"}]} xxx {"writes":[{"path":"a.txt","content":"two"},{"path":"b.txt","content":"y"}]}"#;
        let w = parse_experiment_writes_greedy(raw);
        assert_eq!(w.writes.len(), 2);
        let a = w.writes.iter().find(|x| x.path == "a.txt").unwrap();
        assert_eq!(a.content, "two");
    }
}
