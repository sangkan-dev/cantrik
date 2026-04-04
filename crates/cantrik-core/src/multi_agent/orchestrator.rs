use std::sync::Arc;

use futures_util::future::join_all;
use thiserror::Error;
use tokio::sync::Semaphore;

use crate::config::{
    AppConfig, effective_max_parallel_subagents, effective_max_spawn_depth,
    effective_subagent_summary_max_chars,
};
use crate::llm::{self, LlmError};

use super::parse::{parse_decompose, parse_subagent_summary};
use super::{SubResult, SubTask};

#[derive(Debug, Clone, Default)]
pub struct OrchestratorOptions {
    /// Current spawn depth (CLI entry uses 0).
    pub depth: u8,
    /// Skip worker LLMs and synthesis; print decomposition only.
    pub dry_run: bool,
    pub max_parallel_override: Option<usize>,
}

#[derive(Debug, Error)]
pub enum MultiAgentError {
    #[error("max spawn depth exceeded: depth={depth} max={max}")]
    MaxDepthExceeded { depth: u8, max: u8 },
    #[error("decompose parse: {0}")]
    DecomposeParse(String),
    #[error(transparent)]
    Llm(#[from] LlmError),
}

/// Planner phase: LLM-only decomposition (no filesystem/exec tools).
pub async fn planner_decompose(
    config: &AppConfig,
    goal: &str,
) -> Result<Vec<SubTask>, MultiAgentError> {
    let prompt = format!(
        "You are the Planner agent (read-only analysis only; you cannot run tools or modify files).\n\
Decompose the following goal into 2–6 independent subtasks that can be worked on in parallel.\n\
Return ONLY valid JSON with this shape (no markdown fences):\n\
{{\"subtasks\":[{{\"id\":\"1\",\"instruction\":\"...\"}},{{\"id\":\"2\",\"instruction\":\"...\"}}]}}\n\n\
Goal:\n{goal}"
    );
    let raw = llm::ask_complete_text(config, &prompt).await?;
    let mut tasks =
        parse_decompose(&raw).map_err(|e| MultiAgentError::DecomposeParse(e.to_string()))?;
    ensure_min_subtasks(goal, &mut tasks);
    Ok(tasks)
}

fn ensure_min_subtasks(goal: &str, tasks: &mut Vec<SubTask>) {
    match tasks.len() {
        0 => {
            tasks.push(SubTask {
                id: "1".into(),
                instruction: format!("Analyze the goal and outline key constraints: {goal}"),
            });
            tasks.push(SubTask {
                id: "2".into(),
                instruction: format!("Propose concrete actionable steps: {goal}"),
            });
        }
        1 => {
            let only = tasks[0].instruction.clone();
            tasks.push(SubTask {
                id: "2".into(),
                instruction: format!("Cross-check and add alternatives or risks for: {only}"),
            });
        }
        _ => {}
    }
}

async fn run_one_subtask(
    config: &AppConfig,
    goal: &str,
    task: &SubTask,
    summary_max: usize,
) -> SubResult {
    let prompt = format!(
        "You are sub-agent {} working in isolation (no shared chat history).\n\
Parent goal:\n{goal}\n\nYour assigned instruction:\n{}\n\n\
Reply ONLY with JSON: {{\"summary\":\"brief result for orchestrator\",\"detail\":null}}\n\
Keep summary under ~500 characters of substance; orchestrator will truncate if needed.\n",
        task.id, task.instruction
    );
    match llm::ask_complete_text(config, &prompt).await {
        Ok(raw) => SubResult {
            id: task.id.clone(),
            summary: parse_subagent_summary(&raw, summary_max),
            error: None,
        },
        Err(e) => SubResult {
            id: task.id.clone(),
            summary: String::new(),
            error: Some(e.to_string()),
        },
    }
}

pub async fn run_subtasks_parallel(
    config: &AppConfig,
    goal: &str,
    tasks: &[SubTask],
    max_parallel: usize,
    summary_max: usize,
) -> Vec<SubResult> {
    let sem = Arc::new(Semaphore::new(max_parallel.max(1)));
    let futures: Vec<_> = tasks
        .iter()
        .map(|task| {
            let cfg = config.clone();
            let g = goal.to_string();
            let t = task.clone();
            let sem = sem.clone();
            async move {
                let _permit = match sem.acquire_owned().await {
                    Ok(p) => p,
                    Err(_) => {
                        return SubResult {
                            id: t.id.clone(),
                            summary: String::new(),
                            error: Some("semaphore closed".into()),
                        };
                    }
                };
                run_one_subtask(&cfg, &g, &t, summary_max).await
            }
        })
        .collect();
    join_all(futures).await
}

async fn synthesize_results(
    config: &AppConfig,
    goal: &str,
    results: &[SubResult],
) -> Result<String, LlmError> {
    let mut lines = String::new();
    for r in results {
        let status = if r.error.is_some() { "FAILED" } else { "ok" };
        lines.push_str(&format!(
            "- subtask {} [{}]: {}\n",
            r.id,
            status,
            r.error.as_deref().unwrap_or(&r.summary)
        ));
    }
    let prompt = format!(
        "You are the orchestrator. Combine the following sub-agent outcomes into one coherent answer for the user.\n\
Mention which subtasks failed and what succeeded. User goal:\n{goal}\n\nSub-results:\n{lines}"
    );
    llm::ask_complete_text(config, &prompt).await
}

/// Builder stub: LLM-only note; real Builder would use `cantrik file` / `exec` with `--approve`.
pub async fn run_builder_stub_phase(
    config: &AppConfig,
    goal: &str,
    synthesis: &str,
) -> Result<String, LlmError> {
    let prompt = format!(
        "You represent the Builder role (STUB for Sprint 11).\n\
In production, the Builder would execute changes via `cantrik file write --approve`, `cantrik exec --approve`, etc.\n\
Do not claim any command was run. In 2–4 short sentences, list what you *would* do next for this goal after the draft synthesis.\n\n\
Goal:\n{goal}\n\nDraft synthesis:\n{synthesis}"
    );
    llm::ask_complete_text(config, &prompt).await
}

/// Full pipeline: depth check → planner decompose → (optional) parallel workers → synthesize → builder stub.
pub async fn run_orchestrated(
    config: &AppConfig,
    goal: &str,
    opts: OrchestratorOptions,
) -> Result<String, MultiAgentError> {
    let max_depth = effective_max_spawn_depth(&config.agents);
    if opts.depth >= max_depth {
        return Err(MultiAgentError::MaxDepthExceeded {
            depth: opts.depth,
            max: max_depth,
        });
    }

    let tasks = planner_decompose(config, goal).await?;

    if opts.dry_run {
        let mut out = String::from("Planner decomposition (dry-run; no worker LLMs):\n");
        for t in &tasks {
            out.push_str(&format!("  [{}] {}\n", t.id, t.instruction));
        }
        return Ok(out);
    }

    let max_parallel = opts
        .max_parallel_override
        .unwrap_or_else(|| effective_max_parallel_subagents(&config.agents));
    let summary_max = effective_subagent_summary_max_chars(&config.agents);

    let results = run_subtasks_parallel(config, goal, &tasks, max_parallel, summary_max).await;
    let synthesized = synthesize_results(config, goal, &results).await?;
    let builder = run_builder_stub_phase(config, goal, &synthesized).await?;

    Ok(format!(
        "{synthesized}\n\n--- Builder (stub) ---\n{builder}"
    ))
}
