use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use cantrik_core::config::AppConfig;
use cantrik_core::llm::LlmError;

use super::session_llm;

pub(crate) async fn run(config: &AppConfig, cwd: &Path, prompt: &str) -> ExitCode {
    if prompt.trim().is_empty() {
        eprintln!("ask: empty prompt");
        return ExitCode::from(1);
    }

    let mut stdout = io::stdout().lock();
    let result = session_llm::stream_with_session(cwd, config, prompt, &mut |s| {
        stdout
            .write_all(s.as_bytes())
            .map_err(|e| LlmError::Http(e.to_string()))?;
        stdout.flush().map_err(|e| LlmError::Http(e.to_string()))?;
        Ok(())
    })
    .await;

    match result {
        Ok(()) => {
            let _ = writeln!(stdout);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("ask: {e}");
            ExitCode::from(1)
        }
    }
}
