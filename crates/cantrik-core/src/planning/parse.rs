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
