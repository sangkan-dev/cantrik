//! Structured planning, evaluation, re-planning, and stuck escalation (Sprint 10, PRD §4.4).

mod engine;
mod model;
mod parse;

pub use engine::{FailureRecord, PlanLoopError, PlanOutcome, PlanningLimits, run_plan_loop};
pub use model::{Plan, PlanStep};
pub use parse::{
    ExperimentWrites, extract_json_object, parse_eval_response, parse_experiment_writes,
    parse_experiment_writes_failure_hint_after_empty, parse_experiment_writes_greedy,
    parse_plan_document,
};

#[cfg(test)]
mod tests;
