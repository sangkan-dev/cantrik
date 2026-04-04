use super::engine::{PlanOutcome, PlanningLimits, run_plan_loop};
use super::model::Plan;
use super::parse::{parse_eval_response, parse_experiment_writes, parse_plan_document};

#[test]
fn parse_plan_json_roundtrip() {
    let raw = r#"```json
{"steps":[{"id":"1","description":"Do A","suggested_action":"run tests"}]}
```"#;
    let p = parse_plan_document(raw).expect("parse");
    assert_eq!(p.steps.len(), 1);
    assert_eq!(p.steps[0].id, "1");
}

#[test]
fn parse_eval_bool() {
    let e = parse_eval_response(r#"{"success": false, "notes": "compile error"}"#).expect("e");
    assert!(!e.success);
    assert!(e.notes.contains("compile"));
}

#[test]
fn run_loop_completes_when_eval_always_success() {
    let plan = Plan {
        steps: vec![
            super::model::PlanStep {
                id: "1".into(),
                description: "first".into(),
                suggested_action: None,
            },
            super::model::PlanStep {
                id: "2".into(),
                description: "second".into(),
                suggested_action: None,
            },
        ],
    };
    let limits = PlanningLimits {
        stuck_threshold_attempts: 3,
        max_replan_cycles: 2,
    };
    let out = run_plan_loop("goal", plan, limits, |_prompt| {
        Ok(r#"{"success": true, "notes": "ok"}"#.into())
    })
    .expect("loop");
    assert!(matches!(out, PlanOutcome::Completed));
}

#[test]
fn run_loop_escalates_on_repeated_failure() {
    let plan = Plan::single_manual("only step");
    let limits = PlanningLimits {
        stuck_threshold_attempts: 2,
        max_replan_cycles: 2,
    };
    let out = run_plan_loop("goal", plan, limits, |prompt| {
        if prompt.contains("REVISED") {
            Ok(r#"{"steps":[{"id":"1","description":"retry","suggested_action":null}]}"#.into())
        } else {
            Ok(r#"{"success": false, "notes": "still broken"}"#.into())
        }
    })
    .expect("loop");
    match out {
        PlanOutcome::Escalated { message } => {
            assert!(message.contains("stuck"));
            assert!(message.contains("Yang sudah dicoba"));
        }
        PlanOutcome::Completed => panic!("expected escalation"),
    }
}

#[test]
fn experiment_writes_optional() {
    let ew = parse_experiment_writes("no json here");
    assert!(ew.writes.is_empty());
    let raw = r#"{"writes":[{"path":"a.rs","content":"fn main(){}"}]}"#;
    let ew = parse_experiment_writes(raw);
    assert_eq!(ew.writes.len(), 1);
    assert_eq!(ew.writes[0].path, "a.rs");
}
