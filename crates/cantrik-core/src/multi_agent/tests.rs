use super::parse_decompose;
use super::parse_subagent_summary;
use super::{MultiAgentError, OrchestratorOptions, run_orchestrated};
use crate::config::AppConfig;

#[test]
fn parse_decompose_two_tasks() {
    let raw = r#"{"subtasks":[{"id":"1","instruction":"read auth"},{"id":"2","instruction":"check tests"}]}"#;
    let v = parse_decompose(raw).expect("parse");
    assert_eq!(v.len(), 2);
    assert_eq!(v[0].id, "1");
}

#[test]
fn parse_subagent_summary_json() {
    let s = parse_subagent_summary(r#"{"summary":"hello","detail":null}"#, 100);
    assert_eq!(s, "hello");
}

#[test]
fn parse_subagent_summary_fallback() {
    let s = parse_subagent_summary("plain text only", 10);
    assert!(s.len() <= 10);
}

#[tokio::test]
async fn max_depth_blocks_without_llm() {
    let mut config = AppConfig::default();
    config.agents.max_spawn_depth = Some(2);
    let opts = OrchestratorOptions {
        depth: 5,
        dry_run: false,
        max_parallel_override: None,
        reflect: false,
    };
    let err = run_orchestrated(&config, "any goal", opts)
        .await
        .expect_err("depth");
    assert!(matches!(
        err,
        MultiAgentError::MaxDepthExceeded { depth: 5, max: 2 }
    ));
}
