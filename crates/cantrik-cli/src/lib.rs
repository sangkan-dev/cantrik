//! Cantrik CLI library (thin binary delegates here for tests and structure).

mod cli;
mod commands;
mod repl;

use std::io::{self, IsTerminal, Read};
use std::path::Path;
use std::process::ExitCode;

use cantrik_core::config::{load_merged_config, resolve_config_paths};
use clap::Parser;

pub use cli::{
    Cli, Command, CompletionShell, FileCommand, MacroCommand, SessionCommand, SkillCommand,
};

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
        Some(Command::Skill { sub }) => match sub {
            SkillCommand::Install { name } => commands::skill_cmd::install(&cwd, name),
            SkillCommand::List => commands::skill_cmd::list_registry(),
            SkillCommand::Remove { name } => commands::skill_cmd::remove(&cwd, name),
            SkillCommand::Update { name } => commands::skill_cmd::update(&cwd, name),
        },
        Some(Command::Macro { sub }) => match sub {
            MacroCommand::Record { label } => commands::macro_cmd::record(&cwd, label),
            MacroCommand::Add { args } => commands::macro_cmd::add(&cwd, args),
            MacroCommand::Stop => commands::macro_cmd::stop(&cwd),
            MacroCommand::Run { label } => commands::macro_cmd::run_macro(&cwd, label),
            MacroCommand::List => commands::macro_cmd::list_macros(&cwd),
        },
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
            return commands::ask::run(&config, &cwd, &prompt).await;
        }
        Some(Command::Plan { status, run, task }) => {
            let config = match load_merged_config(&cwd) {
                Ok(config) => config,
                Err(error) => {
                    eprintln!("failed to load config: {error}");
                    return ExitCode::FAILURE;
                }
            };
            let task_line = words_to_line(task);
            return commands::plan::run(&config, &cwd, &task_line, *run, *status).await;
        }
        Some(Command::Experiment { approve, goal }) => {
            let config = match load_merged_config(&cwd) {
                Ok(config) => config,
                Err(error) => {
                    eprintln!("failed to load config: {error}");
                    return ExitCode::FAILURE;
                }
            };
            let g = words_to_line(goal);
            return commands::experiment_cmd::run(&config, &cwd, &g, *approve).await;
        }
        Some(Command::Agents {
            dry_run,
            max_parallel,
            goal,
        }) => {
            let config = match load_merged_config(&cwd) {
                Ok(config) => config,
                Err(error) => {
                    eprintln!("failed to load config: {error}");
                    return ExitCode::FAILURE;
                }
            };
            let g = words_to_line(goal);
            return commands::agents_cmd::run(&config, &cwd, &g, *dry_run, *max_parallel).await;
        }
        Some(Command::Session { sub }) => match sub {
            SessionCommand::List => commands::session_cmd::list_cmd(&cwd).await,
            SessionCommand::Show { limit } => commands::session_cmd::show_cmd(&cwd, *limit).await,
        },
        Some(Command::File { sub }) => {
            let config = match load_merged_config(&cwd) {
                Ok(config) => config,
                Err(error) => {
                    eprintln!("failed to load config: {error}");
                    return ExitCode::FAILURE;
                }
            };
            match sub {
                FileCommand::Read { path } => commands::file_cmd::read_run(&config, &cwd, path),
                FileCommand::Write {
                    path,
                    content_file,
                    approve,
                } => commands::file_cmd::write_run(
                    &config,
                    &cwd,
                    path,
                    content_file.as_deref(),
                    *approve,
                ),
            }
        }
        Some(Command::Exec { approve, argv }) => {
            let config = match load_merged_config(&cwd) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("failed to load config: {e}");
                    return ExitCode::FAILURE;
                }
            };
            commands::exec_cmd::run(&config, &cwd, *approve, argv.clone())
        }
        Some(Command::Rgrep { args }) => {
            let config = match load_merged_config(&cwd) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("failed to load config: {e}");
                    return ExitCode::FAILURE;
                }
            };
            commands::rgrep_cmd::run(&config, &cwd, args.clone())
        }
        Some(Command::Git { args }) => {
            let config = match load_merged_config(&cwd) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("failed to load config: {e}");
                    return ExitCode::FAILURE;
                }
            };
            commands::git_cmd::run(&config, &cwd, args.clone())
        }
        Some(Command::Fetch {
            url,
            approve,
            max_bytes,
        }) => {
            let config = match load_merged_config(&cwd) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("failed to load config: {e}");
                    return ExitCode::FAILURE;
                }
            };
            return commands::fetch_cmd::run(&config, url, *approve, *max_bytes).await;
        }
        Some(Command::Rollback { list, id }) => {
            commands::rollback_cmd::run(&cwd, *list, id.as_deref())
        }
        Some(Command::Index { path, no_vectors }) => {
            let config = match load_merged_config(&cwd) {
                Ok(config) => config,
                Err(error) => {
                    eprintln!("failed to load config: {error}");
                    return ExitCode::FAILURE;
                }
            };
            commands::index::run(&config, path.as_deref(), *no_vectors).await
        }
        Some(Command::Background { no_notify, args }) => {
            commands::background_cmd::run(&cwd, !*no_notify, args).await
        }
        Some(Command::Status { all, limit }) => commands::status_cmd::run(&cwd, *all, *limit).await,
        Some(Command::Daemon { poll_secs }) => commands::daemon_cmd::run(*poll_secs).await,
        Some(Command::Search {
            project,
            limit,
            query,
        }) => {
            let config = match load_merged_config(&cwd) {
                Ok(config) => config,
                Err(error) => {
                    eprintln!("failed to load config: {error}");
                    return ExitCode::FAILURE;
                }
            };
            commands::search::run(&config, project.as_deref(), query, *limit).await
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
            return commands::ask::run(&config, &cwd, &prompt).await;
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
                repl_run(cwd).await
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

    commands::ask::run(&config, &cwd, &text).await
}

async fn repl_run(cwd: std::path::PathBuf) -> ExitCode {
    let config = match load_merged_config(&cwd) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("failed to load config: {error}");
            return ExitCode::FAILURE;
        }
    };
    let handle = tokio::runtime::Handle::current();
    match tokio::task::spawn_blocking(move || repl::run_sync(cwd, config, handle)).await {
        Ok(Ok(())) => ExitCode::SUCCESS,
        Ok(Err(error)) => {
            eprintln!("REPL exited with error: {error}");
            ExitCode::FAILURE
        }
        Err(join_error) => {
            eprintln!("REPL task failed: {join_error}");
            ExitCode::FAILURE
        }
    }
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

    #[test]
    fn parse_search_trailing_query() {
        let cli = Cli::try_parse_from(["cantrik", "search", "hello", "world"]).expect("parse");
        match cli.cmd.expect("cmd") {
            Command::Search {
                query,
                limit,
                project,
            } => {
                assert_eq!(query, vec!["hello", "world"]);
                assert_eq!(limit, 10);
                assert!(project.is_none());
            }
            _ => panic!("expected search"),
        }
    }

    #[test]
    fn parse_index_no_vectors() {
        let cli = Cli::try_parse_from(["cantrik", "index", "--no-vectors"]).expect("parse");
        match cli.cmd.expect("cmd") {
            Command::Index { no_vectors, path } => {
                assert!(no_vectors);
                assert!(path.is_none());
            }
            _ => panic!("expected index"),
        }
    }

    #[test]
    fn parse_background_resume_and_goal() {
        let cli =
            Cli::try_parse_from(["cantrik", "background", "resume", "abc-uuid"]).expect("parse");
        match cli.cmd.expect("cmd") {
            Command::Background { no_notify, args } => {
                assert!(!no_notify);
                assert_eq!(args, vec!["resume", "abc-uuid"]);
            }
            _ => panic!("expected background"),
        }

        let cli = Cli::try_parse_from(["cantrik", "background", "fix", "tests"]).expect("parse");
        match cli.cmd.expect("cmd") {
            Command::Background { args, .. } => assert_eq!(args, vec!["fix", "tests"]),
            _ => panic!("expected background"),
        }
    }
}
