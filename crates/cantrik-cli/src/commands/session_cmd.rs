use std::path::Path;
use std::process::ExitCode;

use cantrik_core::session::{
    connect_pool, latest_summary, list_all_messages_ordered, list_sessions_for_project,
    load_anchors_combined, memory_db_path, message_count, open_or_create_session,
};

pub(crate) async fn list_cmd(cwd: &Path) -> ExitCode {
    match connect_pool().await {
        Ok(pool) => match list_sessions_for_project(&pool, cwd).await {
            Ok(rows) => {
                println!("memory DB: {}", memory_db_path().display());
                if rows.is_empty() {
                    println!("(no sessions for this project yet)");
                }
                for r in rows {
                    println!(
                        "  {}  updated={}  messages={}",
                        r.id, r.updated_at, r.message_count
                    );
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("session list: {e}");
                ExitCode::FAILURE
            }
        },
        Err(e) => {
            eprintln!("session list: cannot open DB: {e}");
            ExitCode::FAILURE
        }
    }
}

pub(crate) async fn show_cmd(cwd: &Path, limit: usize) -> ExitCode {
    let lim = limit.max(1);
    match connect_pool().await {
        Ok(pool) => match open_or_create_session(&pool, cwd).await {
            Ok(sid) => {
                println!("memory DB: {}", memory_db_path().display());
                println!("session: {sid}");
                match message_count(&pool, &sid).await {
                    Ok(n) => println!("message_count: {n}"),
                    Err(e) => eprintln!("count: {e}"),
                }
                let anc = load_anchors_combined(cwd);
                println!("anchors loaded: {} chars", anc.len());
                if let Ok(Some(h)) = latest_summary(&pool, &sid).await {
                    println!(
                        "latest summary covers up to ordinal {}",
                        h.covers_up_to_ordinal
                    );
                }
                let msgs = match list_all_messages_ordered(&pool, &sid).await {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("messages: {e}");
                        return ExitCode::FAILURE;
                    }
                };
                let start = msgs.len().saturating_sub(lim);
                for m in &msgs[start..] {
                    println!("--- {} [{}]", m.role, m.ordinal);
                    println!("{}", m.content);
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("session show: {e}");
                ExitCode::FAILURE
            }
        },
        Err(e) => {
            eprintln!("session show: cannot open DB: {e}");
            ExitCode::FAILURE
        }
    }
}
