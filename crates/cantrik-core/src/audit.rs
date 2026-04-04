//! Append-only human-readable audit log (PRD §5, Sprint 9).

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::session::share_dir;

/// Override path for tests or custom install.
pub const ENV_AUDIT_LOG: &str = "CANTRIK_AUDIT_LOG";

static APPEND_LOCK: Mutex<()> = Mutex::new(());

pub fn audit_log_path() -> PathBuf {
    if let Ok(p) = std::env::var(ENV_AUDIT_LOG) {
        let pb = PathBuf::from(p);
        if let Some(parent) = pb.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        return pb;
    }
    let dir = share_dir();
    let _ = std::fs::create_dir_all(&dir);
    dir.join("audit.log")
}

/// Approximate tokens for cost placeholder (`/cost` later).
pub fn approx_tokens_from_text(s: &str) -> u64 {
    (s.len() as u64).saturating_add(3) / 4
}

/// Stub until pricing table exists (Sprint 14+).
pub fn placeholder_cost_usd(
    _provider: Option<&str>,
    _model: Option<&str>,
    _approx_tokens: u64,
) -> f64 {
    0.0
}

/// One line, newline-terminated; PRD-style.
pub fn append_audit_line(line: &str) -> std::io::Result<()> {
    let _guard = APPEND_LOCK.lock().expect("audit mutex");
    let path = audit_log_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut f = OpenOptions::new().create(true).append(true).open(&path)?;
    f.write_all(line.as_bytes())?;
    if !line.ends_with('\n') {
        f.write_all(b"\n")?;
    }
    f.sync_all()?;
    Ok(())
}

/// Build `[timestamp] WRITE path model=… cost=$… approx_tokens=N`.
pub fn format_write_audit(
    rel_path: &str,
    provider: Option<&str>,
    model: Option<&str>,
    new_contents: &str,
) -> String {
    let ts = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
    let tok = approx_tokens_from_text(new_contents);
    let cost = placeholder_cost_usd(provider, model, tok);
    let model_s = model.unwrap_or("(unset)");
    format!("[{ts}] WRITE  {rel_path}  model={model_s}  approx_tokens={tok}  cost=${cost:.4}\n")
}

pub fn format_exec_audit(program: &str, args: &[String]) -> String {
    let ts = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
    let cmd = std::iter::once(program.to_string())
        .chain(args.iter().cloned())
        .collect::<Vec<_>>()
        .join(" ");
    format!("[{ts}] EXEC   {cmd}  approved_by=user\n")
}

pub fn format_fetch_audit(url: &str, bytes: usize) -> String {
    let ts = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
    format!("[{ts}] FETCH  {url}  bytes={bytes}  approved_by=user\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_twice_preserves_lines() {
        let dir = std::env::temp_dir().join(format!(
            "cantrik-audit-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let log = dir.join("audit.log");
        unsafe {
            std::env::set_var(ENV_AUDIT_LOG, &log);
        }
        append_audit_line("[x] one").unwrap();
        append_audit_line("[x] two").unwrap();
        let s = std::fs::read_to_string(&log).unwrap();
        assert!(s.contains("one") && s.contains("two"));
        unsafe {
            std::env::remove_var(ENV_AUDIT_LOG);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
