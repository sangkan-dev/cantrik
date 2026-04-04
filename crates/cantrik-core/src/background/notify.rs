//! Desktop / webhook / file notifications when a job needs approval (Sprint 12).

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;

/// Resolved notification targets for one approval event.
#[derive(Debug, Clone, Default)]
pub struct NotificationChannels {
    pub desktop: bool,
    pub webhook_url: Option<String>,
    pub flag_path: Option<PathBuf>,
}

#[derive(Serialize)]
struct WebhookBody<'a> {
    event: &'static str,
    job_id: &'a str,
    hint: &'a str,
}

fn applescript_literal(s: &str) -> String {
    s.chars()
        .take(400)
        .map(|c| match c {
            '"' | '\\' => ' ',
            '\n' | '\r' => ' ',
            _ => c,
        })
        .collect()
}

fn try_desktop(title: &str, body: &str) {
    if cfg!(target_os = "macos") {
        let t = applescript_literal(title);
        let b = applescript_literal(body);
        let script = format!("display notification \"{b}\" with title \"{t}\"");
        let _ = Command::new("osascript").args(["-e", &script]).output();
        return;
    }
    if cfg!(target_os = "linux") {
        let _ = Command::new("notify-send").args([title, body]).output();
    }
}

fn try_flag(path: &Path, job_id: &str) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, format!("{job_id}\n"));
}

/// Each channel is best-effort; failures are ignored.
pub async fn notify_approval_needed(job_id: &str, hint: &str, channels: &NotificationChannels) {
    if channels.desktop {
        try_desktop(
            "Cantrik — approval needed",
            &format!("Job {job_id}: {hint}"),
        );
    }

    if let Some(path) = channels.flag_path.as_ref() {
        try_flag(path, job_id);
    }

    if let Some(url) = channels.webhook_url.as_ref() {
        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
        {
            Ok(c) => c,
            Err(_) => return,
        };
        let body = WebhookBody {
            event: "approval_needed",
            job_id,
            hint,
        };
        let _ = client.post(url).json(&body).send().await;
    }
}
