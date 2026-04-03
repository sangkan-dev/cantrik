//! Cantrik CLI library (thin binary delegates here for tests and structure).

mod cli;
mod commands;

use std::io::{self, BufRead, IsTerminal, Read, Write};
use std::path::Path;
use std::process::ExitCode;

use cantrik_core::config::{load_merged_config, resolve_config_paths};
use clap::Parser;

pub use cli::{Cli, Command, CompletionShell};

const STDIN_MAX_BYTES: u64 = 4 * 1024 * 1024;

/// Entry point shared by `main`.
pub async fn run() -> ExitCode {
    let cwd = match std::env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("failed to determine current directory: {error}");
            return ExitCode::FAILURE;
        }
    };

    let cli = Cli::parse();

    if cli.watch {
        eprintln!("--watch is not implemented yet (Sprint 5+ / file watcher).");
        return ExitCode::from(2);
    }

    if cli.from_clipboard {
        eprintln!(
            "--from-clipboard is not implemented yet (requires xclip/wl-paste/pbpaste integration)."
        );
        return ExitCode::from(2);
    }

    if let Some(ref image_path) = cli.image {
        if !image_path.exists() {
            eprintln!("--image: path does not exist: {}", image_path.display());
            return ExitCode::from(2);
        }
        eprintln!(
            "--image: (scaffold) path accepted but vision pipeline is not wired yet — {:?}",
            image_path
        );
        return ExitCode::from(2);
    }

    if cli.global.debug_config {
        let paths = resolve_config_paths(&cwd);
        println!("global config : {}", paths.global.display());
        println!("project config: {}", paths.project.display());
    }

    match &cli.cmd {
        Some(Command::Completions { shell }) => commands::completions::run(*shell),
        Some(Command::Doctor) => commands::doctor::run(&cwd),
        Some(Command::Ask { query }) => {
            let config = match load_merged_config(&cwd) {
                Ok(config) => config,
                Err(error) => {
                    eprintln!("failed to load config: {error}");
                    return ExitCode::FAILURE;
                }
            };
            let prompt = words_to_line(query);
            commands::ask::run(&config, &prompt)
        }
        Some(Command::Plan { task }) => {
            let config = match load_merged_config(&cwd) {
                Ok(config) => config,
                Err(error) => {
                    eprintln!("failed to load config: {error}");
                    return ExitCode::FAILURE;
                }
            };
            let task_line = words_to_line(task);
            commands::plan::run(&config, &task_line)
        }
        Some(Command::Index { path }) => {
            let config = match load_merged_config(&cwd) {
                Ok(config) => config,
                Err(error) => {
                    eprintln!("failed to load config: {error}");
                    return ExitCode::FAILURE;
                }
            };
            commands::index::run(&config, path.as_deref())
        }
        Some(Command::External(extra)) => {
            let config = match load_merged_config(&cwd) {
                Ok(config) => config,
                Err(error) => {
                    eprintln!("failed to load config: {error}");
                    return ExitCode::FAILURE;
                }
            };
            let prompt = os_string_args_to_line(extra);
            commands::ask::run(&config, &prompt)
        }
        None => {
            // `cantrik --debug-config` alone: match legacy behaviour (resolve + load + exit).
            if cli.global.debug_config {
                return match load_merged_config(&cwd) {
                    Ok(_) => ExitCode::SUCCESS,
                    Err(error) => {
                        eprintln!("failed to load config: {error}");
                        ExitCode::FAILURE
                    }
                };
            }

            if io::stdin().is_terminal() {
                repl_placeholder().await
            } else {
                stdin_pipe_ask(&cwd).await
            }
        }
    }
}

fn words_to_line(words: &[String]) -> String {
    words.join(" ")
}

fn os_string_args_to_line(parts: &[std::ffi::OsString]) -> String {
    parts
        .iter()
        .map(|s| s.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ")
}

fn read_stdin_limited(max_bytes: u64) -> io::Result<String> {
    let mut buf = Vec::new();
    let mut limited = io::stdin().take(max_bytes.saturating_add(1));
    limited.read_to_end(&mut buf)?;
    if buf.len() as u64 > max_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("stdin larger than {max_bytes} bytes"),
        ));
    }
    String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.utf8_error()))
}

async fn stdin_pipe_ask(cwd: &Path) -> ExitCode {
    let cwd = cwd.to_path_buf();
    let text = match tokio::task::spawn_blocking(move || read_stdin_limited(STDIN_MAX_BYTES)).await
    {
        Ok(Ok(text)) => text,
        Ok(Err(error)) => {
            eprintln!("failed to read stdin: {error}");
            return ExitCode::FAILURE;
        }
        Err(join_error) => {
            eprintln!("stdin task failed: {join_error}");
            return ExitCode::FAILURE;
        }
    };

    let config = match load_merged_config(&cwd) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("failed to load config: {error}");
            return ExitCode::FAILURE;
        }
    };

    commands::ask::run(&config, &text)
}

async fn repl_placeholder() -> ExitCode {
    let result = tokio::task::spawn_blocking(repl_sync).await;
    match result {
        Ok(code) => code,
        Err(e) => {
            eprintln!("REPL task failed: {e}");
            ExitCode::FAILURE
        }
    }
}

fn repl_sync() -> ExitCode {
    println!(
        "Cantrik REPL (placeholder). Type 'exit' or 'quit', or Ctrl+D to leave. Full TUI arrives in Sprint 4."
    );
    let stdin = io::stdin();
    let mut line = String::new();
    loop {
        line.clear();
        print!("cantrik> ");
        let _ = io::stdout().flush();
        let n = match stdin.lock().read_line(&mut line) {
            Ok(n) => n,
            Err(error) => {
                eprintln!("failed to read line: {error}");
                return ExitCode::FAILURE;
            }
        };
        if n == 0 {
            println!();
            break;
        }
        let trimmed = line.trim();
        if trimmed.eq_ignore_ascii_case("exit") || trimmed.eq_ignore_ascii_case("quit") {
            break;
        }
        if !trimmed.is_empty() {
            println!("(REPL placeholder) you said: {trimmed}");
        }
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_help_builds() {
        Cli::command().debug_assert();
    }

    #[test]
    fn parse_ask_trailing_args() {
        let cli = Cli::try_parse_from(["cantrik", "ask", "hello", "world"]).expect("parse");
        assert!(cli.cmd.is_some());
        match cli.cmd.unwrap() {
            Command::Ask { query } => assert_eq!(query, vec!["hello", "world"]),
            _ => panic!("wrong subcommand"),
        }
    }

    #[test]
    fn parse_external_routes_to_freeform() {
        let cli = Cli::try_parse_from(["cantrik", "explain", "this", "repo"]).expect("parse");
        match cli.cmd.unwrap() {
            Command::External(parts) => {
                assert_eq!(parts.len(), 3);
            }
            _ => panic!("expected external"),
        }
    }

    #[test]
    fn global_debug_on_subcommand() {
        let cli = Cli::try_parse_from(["cantrik", "--debug-config", "doctor"]).expect("parse");
        assert!(cli.global.debug_config);
        assert!(matches!(cli.cmd, Some(Command::Doctor)));
    }
}
