//! Multi-agent orchestration (Sprint 11, PRD §4.2).

mod orchestrator;
mod parse;

pub use orchestrator::{
    MultiAgentError, OrchestratorOptions, planner_decompose, run_builder_stub_phase,
    run_orchestrated,
};
pub use parse::{MultiAgentParseError, parse_decompose, parse_subagent_summary};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubTask {
    pub id: String,
    pub instruction: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubResult {
    pub id: String,
    pub summary: String,
    pub error: Option<String>,
}

#[cfg(test)]
mod tests;
