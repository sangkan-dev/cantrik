//! Companion process: watch `~/.local/share/cantrik/approval-pending.flag` (same default as core background jobs).
//! Sprint 19 MVP — polling + desktop notification. A full Tauri tray shell can embed this logic later.

use std::path::PathBuf;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let flag_path = approval_flag_path();
    eprintln!("cantrik-tray: watching {}", flag_path.display());
    let mut was_present = flag_path.exists();
    loop {
        let present = flag_path.exists();
        if present && !was_present {
            let body = std::fs::read_to_string(&flag_path)
                .map(|s| format!("Job id / hint: {}", s.trim()))
                .unwrap_or_else(|_| "Open a terminal and use `cantrik background` / docs.".into());
            if let Err(e) = notify_rust::Notification::new()
                .summary("Cantrik — approval needed")
                .body(&body)
                .show()
            {
                eprintln!("cantrik-tray: notification error: {e}");
            }
        }
        was_present = present;
        tokio::time::sleep(Duration::from_secs(8)).await;
    }
}

fn approval_flag_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cantrik")
        .join("approval-pending.flag")
}
