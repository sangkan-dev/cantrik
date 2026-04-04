use thiserror::Error;

use super::model::{Plan, PlanStep};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanningLimits {
    pub stuck_threshold_attempts: u32,
    pub max_replan_cycles: u32,
}

impl Default for PlanningLimits {
    fn default() -> Self {
        Self {
            stuck_threshold_attempts: 3,
            max_replan_cycles: 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureRecord {
    pub step_id: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanOutcome {
    Completed,
    Escalated { message: String },
}

#[derive(Debug, Error)]
pub enum PlanLoopError {
    #[error("llm: {0}")]
    Llm(String),
}

/// Synchronous plan → act (simulated summary) → evaluate loop with injectable LLM `complete`.
///
/// `act_summary` builds the text passed to the evaluator (MVP: derived from step + optional hint).
pub fn run_plan_loop(
    goal: &str,
    initial_plan: Plan,
    limits: PlanningLimits,
    mut complete: impl FnMut(&str) -> Result<String, PlanLoopError>,
) -> Result<PlanOutcome, PlanLoopError> {
    let mut plan = initial_plan;
    let mut step_index: usize = 0;
    let mut replan_cycles: u32 = 0;
    let mut consecutive_failures: u32 = 0;
    let mut failures: Vec<FailureRecord> = Vec::new();

    loop {
        if step_index >= plan.steps.len() {
            return Ok(PlanOutcome::Completed);
        }

        let step = plan.steps[step_index].clone();
        let act_summary = build_act_summary(goal, &plan, &step);

        let eval_prompt = format!(
            r#"You evaluate whether a single plan step was successfully completed in spirit (MVP: we only have a short activity summary, not live tool output).

Goal: {goal}

Full plan (JSON-ish list):
{plan_lines}

Current step id={id}
Description: {desc}
Suggested action (optional): {sug}

Activity / outcome summary for this step:
{act}

Reply with ONLY a JSON object: {{"success": true or false, "notes": "brief reason"}}"#,
            plan_lines = plan_lines(&plan.steps),
            id = step.id,
            desc = step.description,
            sug = step.suggested_action.as_deref().unwrap_or("(none)"),
            act = act_summary,
        );

        let eval_raw = complete(&eval_prompt)?;
        let eval = super::parse::parse_eval_response(&eval_raw).map_err(|e| {
            PlanLoopError::Llm(format!(
                "eval JSON parse failed: {e}; raw: {}",
                trunc(&eval_raw)
            ))
        })?;

        if eval.success {
            consecutive_failures = 0;
            step_index += 1;
            continue;
        }

        consecutive_failures += 1;
        failures.push(FailureRecord {
            step_id: step.id.clone(),
            summary: eval.notes.clone(),
        });

        if consecutive_failures >= limits.stuck_threshold_attempts {
            return Ok(PlanOutcome::Escalated {
                message: escalation_message(goal, &failures, &eval.notes),
            });
        }

        if replan_cycles >= limits.max_replan_cycles {
            return Ok(PlanOutcome::Escalated {
                message: escalation_message(goal, &failures, &eval.notes),
            });
        }

        replan_cycles += 1;
        consecutive_failures = 0;
        step_index = 0;

        let failures_txt = failures
            .iter()
            .map(|f| format!("- step {}: {}", f.step_id, f.summary))
            .collect::<Vec<_>>()
            .join("\n");

        let replan_prompt = format!(
            r#"The following plan did not succeed when evaluated. Produce a REVISED plan as JSON only.

Goal: {goal}

Failed attempts / notes:
{failures_txt}

Previous plan steps:
{plan_lines}

Return JSON exactly in this shape:
{{"steps":[{{"id":"1","description":"...","suggested_action":"optional"}}]}}"#,
            plan_lines = plan_lines(&plan.steps),
        );

        let replan_raw = complete(&replan_prompt)?;
        plan = super::parse::parse_plan_document(&replan_raw).unwrap_or_else(|_| {
            Plan::single_manual(format!(
                "Re-plan parse failed; manual follow-up. Last model output:\n{}",
                trunc(&replan_raw)
            ))
        });
    }
}

fn plan_lines(steps: &[PlanStep]) -> String {
    steps
        .iter()
        .map(|s| {
            format!(
                "- [{}] {} (suggested: {})",
                s.id,
                s.description,
                s.suggested_action.as_deref().unwrap_or("-")
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn build_act_summary(goal: &str, _plan: &Plan, step: &PlanStep) -> String {
    format!(
        "(MVP simulated act) User or agent aimed to complete this step toward goal {:?}. \
         Step {:?}: {}. Assume typical progress unless contradicted.",
        trunc(goal),
        step.id,
        step.description
    )
}

fn trunc(s: &str) -> String {
    const MAX: usize = 800;
    if s.len() <= MAX {
        s.to_string()
    } else {
        format!("{}…", &s[..MAX])
    }
}

pub(crate) fn escalation_message(
    goal: &str,
    failures: &[FailureRecord],
    last_notes: &str,
) -> String {
    let mut out = String::new();
    out.push_str("⚠  Cantrik stuck: planning loop could not validate progress.\n\n");
    out.push_str("Goal:\n");
    out.push_str(goal);
    out.push_str("\n\nYang sudah dicoba:\n");
    for (i, f) in failures.iter().enumerate() {
        out.push_str(&format!("{}. Step {} → {}\n", i + 1, f.step_id, f.summary));
    }
    out.push_str("\nCatatan evaluator terakhir:\n");
    out.push_str(last_notes);
    out.push_str("\n\nButuh bantuan: tinjau log di atas atau sederhanakan task.\n");
    out
}
