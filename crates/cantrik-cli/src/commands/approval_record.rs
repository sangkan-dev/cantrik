//! Record Adaptive Begawan rows when the user explicitly approves tool use (Sprint 19).

use std::path::Path;

use cantrik_core::config::{AppConfig, effective_adaptive_begawan};
use cantrik_core::session::{connect_pool, record_approval_memory, session_project_fingerprint};

pub async fn record_if_adaptive(
    cwd: &Path,
    app: &AppConfig,
    tool_id: &str,
    approved: bool,
    summary: &str,
) {
    if !effective_adaptive_begawan(&app.memory) {
        return;
    }
    let Ok(pool) = connect_pool().await else {
        return;
    };
    let fp = session_project_fingerprint(cwd, app);
    let sum: String = summary.chars().take(400).collect();
    let _ = record_approval_memory(&pool, &fp, tool_id, approved, &sum).await;
}
