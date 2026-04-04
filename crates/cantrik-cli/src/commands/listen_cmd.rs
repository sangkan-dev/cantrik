//! `cantrik listen` — STT + `ask` (Sprint 18, PRD §4.26).

use std::fs;
use std::path::Path;
use std::process::ExitCode;

use cantrik_core::config::{AppConfig, effective_voice_enabled};
use cantrik_core::voice;

use super::ask;

pub async fn run(
    config: &AppConfig,
    cwd: &Path,
    file: Option<std::path::PathBuf>,
    raw_text: Option<String>,
) -> ExitCode {
    if let Some(t) = raw_text {
        let line = t.trim();
        if line.is_empty() {
            eprintln!("listen: --raw-text is empty");
            return ExitCode::from(2);
        }
        return ask::run(config, cwd, line).await;
    }

    let Some(path) = file else {
        eprintln!("listen: use --file AUDIO.wav (Ollama /api/transcribe) or --raw-text \"...\"");
        return ExitCode::from(2);
    };

    if !effective_voice_enabled(&config.ui) {
        eprintln!("listen: enable `[ui] voice_enabled = true` in cantrik.toml (opt-in).");
        return ExitCode::from(2);
    }

    let bytes = match fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("listen: read {}: {e}", path.display());
            return ExitCode::FAILURE;
        }
    };
    let fname = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("audio.bin");

    let text = match voice::transcribe_ollama(config, bytes, fname).await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("listen: {e}");
            return ExitCode::FAILURE;
        }
    };

    ask::run(config, cwd, &text).await
}
