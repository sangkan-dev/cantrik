//! `cantrik sync` — rsync over ssh using `[remote_exec]` (dry-run by default).

use std::path::Path;
use std::process::ExitCode;

use cantrik_core::config::{AppConfig, remote_ssh_destination};

fn sh_quote_for_ssh_e(s: &str) -> String {
    if s.is_empty() {
        return "''".into();
    }
    if s.chars()
        .all(|c| c.is_ascii_alphanumeric() || "/._+-=@:,^%".contains(c))
    {
        return s.to_string();
    }
    format!("'{}'", s.replace('\'', "'\"'\"'"))
}

/// Single string for `rsync -e '<here>'` (bourne-like quoting).
fn rsync_ssh_e_shell(config: &AppConfig) -> String {
    let mut parts = vec![
        "ssh".to_string(),
        "-o".into(),
        "BatchMode=yes".into(),
        "-o".into(),
        "ConnectTimeout=15".into(),
    ];
    if let Some(ref id) = config.remote_exec.identity_file
        && !id.trim().is_empty()
    {
        parts.push("-i".into());
        parts.push(sh_quote_for_ssh_e(id.trim()));
    }
    for x in &config.remote_exec.extra_ssh_args {
        parts.push(sh_quote_for_ssh_e(x));
    }
    parts.join(" ")
}

pub(crate) fn run(config: &AppConfig, cwd: &Path, approve: bool, src: &Path) -> ExitCode {
    let Some(dest) = remote_ssh_destination(&config.remote_exec) else {
        eprintln!(
            "sync: set [remote_exec].host in cantrik.toml (see docs/rfc-hybrid-ssh-executor.md)."
        );
        return ExitCode::FAILURE;
    };
    let Some(ref remote_dir) = config.remote_exec.sync_remote_dir else {
        eprintln!(
            "sync: set [remote_exec].sync_remote_dir (remote path) for rsync destination."
        );
        return ExitCode::FAILURE;
    };
    let remote_dir = remote_dir.trim();
    if remote_dir.is_empty() {
        eprintln!("sync: [remote_exec].sync_remote_dir must be non-empty.");
        return ExitCode::FAILURE;
    }

    let abs_src = cwd.join(src);
    let abs_src = match abs_src.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("sync: cannot resolve local src {:?}: {e}", abs_src);
            return ExitCode::FAILURE;
        }
    };

    let mut src_arg = abs_src.display().to_string();
    if abs_src.is_dir() && !src_arg.ends_with('/') {
        src_arg.push('/');
    }

    let rd = remote_dir.trim_end_matches('/');
    let remote_target = format!("{dest}:{rd}/");

    let ssh_e = rsync_ssh_e_shell(config);
    let preview = {
        let mut w = vec![
            "rsync".to_string(),
            "-az".to_string(),
            "--delete".to_string(),
            "-e".to_string(),
            ssh_e.clone(),
            src_arg.clone(),
            remote_target.clone(),
        ];
        if !approve {
            w.insert(2, "--dry-run".into());
        }
        shell_words_join(&w)
    };
    if !approve {
        eprintln!("sync (dry-run): would run:\n  {preview}");
        eprintln!("sync: pass --approve to run rsync (still review remote impact).");
        return ExitCode::SUCCESS;
    }

    let mut cmd = std::process::Command::new("rsync");
    cmd.arg("-az").arg("--delete").arg("-e").arg(&ssh_e);
    cmd.arg(&src_arg).arg(&remote_target);
    cmd.current_dir(cwd);
    match cmd.status() {
        Ok(s) => ExitCode::from(s.code().map(|c| c as u8).unwrap_or(1)),
        Err(e) => {
            eprintln!("sync: failed to spawn rsync: {e}");
            ExitCode::FAILURE
        }
    }
}

fn shell_words_join(words: &[String]) -> String {
    words
        .iter()
        .map(|w| {
            if w.contains(' ') || w.contains('\'') {
                format!("'{}'", w.replace('\'', "'\"'\"'"))
            } else {
                w.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use cantrik_core::config::RemoteExecConfig;
    use std::path::Path;

    #[test]
    fn sync_requires_remote_dir() {
        let mut cfg = AppConfig::default();
        cfg.remote_exec = RemoteExecConfig {
            host: Some("h".into()),
            user: Some("u".into()),
            identity_file: None,
            extra_ssh_args: vec![],
            sync_remote_dir: None,
        };
        let cwd = std::env::current_dir().unwrap();
        let code = run(&cfg, &cwd, false, Path::new("."));
        assert_eq!(code, std::process::ExitCode::FAILURE);
    }

    #[test]
    fn sync_dry_run_ok_without_spawning_rsync() {
        let mut cfg = AppConfig::default();
        cfg.remote_exec = RemoteExecConfig {
            host: Some("build.example.com".into()),
            user: Some("builder".into()),
            identity_file: None,
            extra_ssh_args: vec![],
            sync_remote_dir: Some("/home/builder/app".into()),
        };
        let cwd = std::env::temp_dir();
        let code = run(&cfg, &cwd, false, Path::new("."));
        assert_eq!(code, std::process::ExitCode::SUCCESS);
    }
}
