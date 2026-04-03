use std::io::{self, Write};
use std::process::ExitCode;

use cantrik_core::config::AppConfig;
use cantrik_core::llm::{self, LlmError};

pub(crate) async fn run(config: &AppConfig, task: &str) -> ExitCode {
    if task.trim().is_empty() {
        eprintln!("plan: empty task");
        return ExitCode::from(1);
    }

    let prompt = format!(
        "Produce a concise, ordered step-by-step plan (no execution yet) for the following goal:\n\n{}",
        task.trim()
    );

    let mut stdout = io::stdout().lock();
    let result = llm::ask_stream_chunks(config, &prompt, &mut |s| {
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
            eprintln!("plan: {e}");
            ExitCode::from(1)
        }
    }
}
