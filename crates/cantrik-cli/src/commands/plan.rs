use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use cantrik_core::config::{AppConfig, effective_max_replan_cycles, effective_stuck_threshold};
use cantrik_core::planning::{
    Plan, PlanLoopError, PlanOutcome, PlanningLimits, parse_plan_document, run_plan_loop,
};
use serde::{Deserialize, Serialize};

use super::session_llm;

const PLAN_STATE_FILE: &str = "plan-state.json";

#[derive(Serialize, Deserialize)]
struct PlanStateFile {
    schema_version: u32,
    goal: String,
    plan: Plan,
}

fn plan_state_path(cwd: &Path) -> PathBuf {
    cwd.join(".cantrik").join(PLAN_STATE_FILE)
}

fn write_plan_run_summary(cwd: &Path, goal: &str, outcome: &str, detail: Option<&str>) {
    let dir = cwd.join(".cantrik");
    let _ = fs::create_dir_all(&dir);
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let v = serde_json::json!({
        "schema_version": 1u32,
        "finished_at_unix": ts,
        "goal": goal,
        "outcome": outcome,
        "detail": detail,
    });
    if let Ok(text) = serde_json::to_string_pretty(&v) {
        let _ = fs::write(dir.join("plan-run-summary.json"), text);
    }
}

fn save_plan_state(cwd: &Path, goal: &str, plan: &Plan) -> Result<(), String> {
    let dir = cwd.join(".cantrik");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let state = PlanStateFile {
        schema_version: 1,
        goal: goal.to_string(),
        plan: plan.clone(),
    };
    let text = serde_json::to_string_pretty(&state).map_err(|e| e.to_string())?;
    fs::write(plan_state_path(cwd), text).map_err(|e| e.to_string())
}

fn print_plan(goal: &str, plan: &Plan) {
    println!("Goal: {goal}");
    println!("Steps:");
    for s in &plan.steps {
        println!("  [{}] {}", s.id, s.description);
        if let Some(a) = &s.suggested_action {
            println!("      (hint) {a}");
        }
    }
}

pub(crate) async fn run(
    config: &AppConfig,
    cwd: &Path,
    task: &str,
    run_loop: bool,
    status_only: bool,
) -> ExitCode {
    if status_only {
        let path = plan_state_path(cwd);
        if !path.is_file() {
            println!("plan: no saved state at {}", path.display());
            println!("hint: run `cantrik plan \"your task\"` first.");
            return ExitCode::SUCCESS;
        }
        let text = match fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("plan --status: {e}");
                return ExitCode::FAILURE;
            }
        };
        let state: PlanStateFile = match serde_json::from_str(&text) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("plan --status: corrupt state: {e}");
                return ExitCode::FAILURE;
            }
        };
        print_plan(&state.goal, &state.plan);
        return ExitCode::SUCCESS;
    }

    if task.trim().is_empty() {
        eprintln!("plan: empty task (use `cantrik plan --status` to show saved plan)");
        return ExitCode::from(1);
    }

    let goal = task.trim();
    let gen_prompt = format!(
        "Output ONLY valid JSON (no markdown fences, no commentary before or after) with this exact shape:\n\
{{\"steps\":[{{\"id\":\"1\",\"description\":\"string\",\"suggested_action\":null}}]}}\n\n\
Use short ids (1,2,3). suggested_action is optional string or null.\n\n\
Task to plan:\n{goal}"
    );

    let raw = match session_llm::complete_with_session(cwd, config, &gen_prompt).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("plan: {e}");
            return ExitCode::from(1);
        }
    };

    let plan = parse_plan_document(&raw).unwrap_or_else(|_| {
        eprintln!("plan: (warn) could not parse JSON plan; using single manual step.");
        Plan::single_manual(goal.to_string())
    });

    if let Err(e) = save_plan_state(cwd, goal, &plan) {
        eprintln!("plan: could not save state: {e}");
    }

    print_plan(goal, &plan);

    if !run_loop {
        return ExitCode::SUCCESS;
    }

    let limits = PlanningLimits {
        stuck_threshold_attempts: effective_stuck_threshold(&config.planning),
        max_replan_cycles: effective_max_replan_cycles(&config.planning),
    };

    let handle = tokio::runtime::Handle::current();
    let cwd_buf = cwd.to_path_buf();
    let cfg = config.clone();
    let goal_owned = goal.to_string();

    let outcome = tokio::task::block_in_place(|| {
        run_plan_loop(&goal_owned, plan, limits, |prompt| {
            handle
                .block_on(session_llm::complete_with_session(&cwd_buf, &cfg, prompt))
                .map_err(|e| PlanLoopError::Llm(e.to_string()))
        })
    });

    match &outcome {
        Ok(PlanOutcome::Completed) => {
            write_plan_run_summary(cwd, &goal_owned, "completed", None);
            println!("\nplan --run: all steps evaluated successfully (per model).");
            ExitCode::SUCCESS
        }
        Ok(PlanOutcome::Escalated { message }) => {
            write_plan_run_summary(cwd, &goal_owned, "escalated", Some(message.as_str()));
            eprintln!("\n{message}");
            ExitCode::from(1)
        }
        Err(e) => {
            let msg = e.to_string();
            write_plan_run_summary(cwd, &goal_owned, "error", Some(msg.as_str()));
            eprintln!("plan --run: {e}");
            ExitCode::from(1)
        }
    }
}
