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
    Cli, Command, CompletionShell, FileCommand, MacroCommand, McpCommand, PrCommand, ReplayCommand,
    SessionCommand, SkillCommand, TeachFormatArg, VisualizeCliKind, WebCommand, WorkspaceCommand,
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
        Some(Command::Lsp) => match commands::lsp_cmd::run().await {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("cantrik lsp: {e}");
                ExitCode::FAILURE
            }
        },
        Some(Command::Cost { session_only }) => commands::cost_cmd::run(&cwd, *session_only).await,
        Some(Command::Serve { mcp }) => {
            if !*mcp {
                eprintln!(
                    "cantrik serve: use --mcp for MCP over stdio (cwd = project root for config)."
                );
                return ExitCode::from(2);
            }
            match load_merged_config(&cwd) {
                Ok(config) => commands::serve_mcp::run_mcp_stdio(config).await,
                Err(error) => {
                    eprintln!("failed to load config: {error}");
                    ExitCode::FAILURE
                }
            }
        }
        Some(Command::Mcp { sub }) => match sub {
            McpCommand::Call { server, tool, json } => {
                return commands::mcp_client_cmd::run_call(
                    &cwd,
                    server.clone(),
                    tool.clone(),
                    json.clone(),
                )
                .await;
            }
        },
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
        Some(Command::Diff {
            staged,
            text_only,
            conflicts,
        }) => commands::diff_cmd::run(&cwd, *staged, *text_only, *conflicts).await,
        Some(Command::Handoff { message }) => {
            commands::collab_cmd::handoff(&cwd, message.clone()).await
        }
        Some(Command::Export { output }) => {
            commands::collab_cmd::export_context(&cwd, output).await
        }
        Some(Command::Import {
            input,
            seed_session,
        }) => commands::collab_cmd::import_context(&cwd, input, *seed_session).await,
        Some(Command::Replay { sub }) => match sub {
            ReplayCommand::Export { output } => {
                commands::collab_cmd::replay_export(&cwd, output).await
            }
            ReplayCommand::Play { file } => commands::collab_cmd::replay_play(file),
        },
        Some(Command::Workspace { sub }) => commands::workspace_cmd::run(&cwd, sub).await,
        Some(Command::Pr { sub }) => commands::pr_cmd::run_standalone(&cwd, sub),
        Some(Command::Review { worktree, soft }) => {
            commands::review_cmd::run(&cwd, *worktree, *soft).await
        }
        Some(Command::Explain { path, why, line }) => {
            let config = match load_merged_config(&cwd) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("failed to load config: {e}");
                    return ExitCode::FAILURE;
                }
            };
            commands::intelligence_cmd::run_explain(&config, &cwd, path, *line, *why).await
        }
        Some(Command::Teach { output_dir, format }) => {
            let config = match load_merged_config(&cwd) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("failed to load config: {e}");
                    return ExitCode::FAILURE;
                }
            };
            let fmt = match format {
                cli::TeachFormatArg::Wiki => commands::intelligence_cmd::TeachFormat::Wiki,
                cli::TeachFormatArg::Markdown => commands::intelligence_cmd::TeachFormat::Markdown,
            };
            commands::intelligence_cmd::run_teach(&config, &cwd, output_dir.as_deref(), fmt).await
        }
        Some(Command::Why {
            crate_name,
            synthesize,
        }) => {
            let config = match load_merged_config(&cwd) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("failed to load config: {e}");
                    return ExitCode::FAILURE;
                }
            };
            commands::intelligence_cmd::run_why(&config, &cwd, crate_name, *synthesize).await
        }
        Some(Command::Upgrade) => {
            let config = match load_merged_config(&cwd) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("failed to load config: {e}");
                    return ExitCode::FAILURE;
                }
            };
            commands::intelligence_cmd::run_upgrade(&config, &cwd).await
        }
        Some(Command::Audit) => {
            let config = match load_merged_config(&cwd) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("failed to load config: {e}");
                    return ExitCode::FAILURE;
                }
            };
            commands::intelligence_cmd::run_audit(&config, &cwd)
        }
        Some(Command::Fix {
            issue_url,
            fetch,
            approve,
            run_agents,
            run_experiment,
        }) => {
            let config = match load_merged_config(&cwd) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("failed to load config: {e}");
                    return ExitCode::FAILURE;
                }
            };
            commands::fix_cmd::run(
                &cwd,
                &config,
                issue_url,
                *fetch,
                *approve,
                *run_agents,
                *run_experiment,
            )
            .await
        }
        Some(Command::Web { sub }) => commands::web_cmd::run(&cwd, sub).await,
        Some(Command::Visualize { mode, output }) => {
            commands::visualize_cmd::run(&cwd, *mode, output.clone())
        }
        Some(Command::Listen { file, raw_text }) => {
            let config = match load_merged_config(&cwd) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("failed to load config: {e}");
                    return ExitCode::FAILURE;
                }
            };
            commands::listen_cmd::run(&config, &cwd, file.clone(), raw_text.clone()).await
        }
        Some(Command::Init { template, path }) => {
            let root = if path.is_absolute() {
                path.clone()
            } else {
                cwd.join(path)
            };
            commands::init_cmd::run(&root, template.as_str())
        }
        Some(Command::Doctor) => commands::doctor::run(&cwd),
        Some(Command::Health {
            soft,
            no_clippy,
            no_test,
            timeout_sec,
            tree,
            outdated,
            coverage,
            deny,
            audit,
            sarif,
        }) => {
            commands::health::run(
                &cwd,
                &commands::health::HealthCli {
                    soft: *soft,
                    no_clippy: *no_clippy,
                    no_test: *no_test,
                    timeout_sec: *timeout_sec,
                    tree: *tree,
                    outdated: *outdated,
                    coverage: *coverage,
                    deny: *deny,
                    audit: *audit,
                    sarif: *sarif,
                },
            )
            .await
        }
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
            reflect,
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
            return commands::agents_cmd::run(&config, &cwd, &g, *dry_run, *max_parallel, *reflect)
                .await;
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
                } => {
                    commands::file_cmd::write_run(
                        &config,
                        &cwd,
                        path,
                        content_file.as_deref(),
                        *approve,
                    )
                    .await
                }
            }
        }
        Some(Command::Exec {
            approve,
            remote,
            argv,
        }) => {
            let config = match load_merged_config(&cwd) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("failed to load config: {e}");
                    return ExitCode::FAILURE;
                }
            };
            commands::exec_cmd::run(&config, &cwd, *approve, *remote, argv.clone()).await
        }
        Some(Command::Sync { approve, src }) => {
            let config = match load_merged_config(&cwd) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("failed to load config: {e}");
                    return ExitCode::FAILURE;
                }
            };
            commands::sync_cmd::run(&config, &cwd, *approve, src)
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
        Some(Command::Status {
            all,
            limit,
            json,
            write_harness_summary,
        }) => commands::status_cmd::run(&cwd, *all, *limit, *json, *write_harness_summary).await,
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
        // Must not use a real subcommand name (e.g. `explain` is Sprint 17).
        let cli =
            Cli::try_parse_from(["cantrik", "not_a_subcommand", "this", "repo"]).expect("parse");
        match cli.cmd.unwrap() {
            Command::External(parts) => {
                assert_eq!(parts.len(), 3);
            }
            _ => panic!("expected external"),
        }
    }

    #[test]
    fn parse_explain_file_line_why() {
        let cli = Cli::try_parse_from([
            "cantrik",
            "explain",
            "crates/foo/src/lib.rs",
            "--why",
            "--line",
            "10",
        ])
        .expect("parse");
        match cli.cmd.expect("cmd") {
            Command::Explain { path, why, line } => {
                assert_eq!(path, std::path::PathBuf::from("crates/foo/src/lib.rs"));
                assert!(why);
                assert_eq!(line, Some(10));
            }
            _ => panic!("expected explain"),
        }
    }

    #[test]
    fn parse_teach_output_dir_format() {
        let cli = Cli::try_parse_from([
            "cantrik",
            "teach",
            "--output-dir",
            ".cantrik/docs",
            "--format",
            "wiki",
        ])
        .expect("parse");
        match cli.cmd.expect("cmd") {
            Command::Teach { output_dir, format } => {
                assert_eq!(output_dir, Some(std::path::PathBuf::from(".cantrik/docs")));
                assert_eq!(format, TeachFormatArg::Wiki);
            }
            _ => panic!("expected teach"),
        }
    }

    #[test]
    fn parse_visualize_and_listen() {
        let cli = Cli::try_parse_from(["cantrik", "visualize", "dependencies", "-o", "out.mmd"])
            .expect("parse");
        match cli.cmd.expect("cmd") {
            Command::Visualize { mode, output } => {
                assert_eq!(mode, VisualizeCliKind::Dependencies);
                assert_eq!(output, Some(std::path::PathBuf::from("out.mmd")));
            }
            _ => panic!("expected visualize"),
        }
        let cli2 =
            Cli::try_parse_from(["cantrik", "listen", "--raw-text", "hello world"]).expect("parse");
        match cli2.cmd.expect("cmd") {
            Command::Listen { raw_text, .. } => {
                assert_eq!(raw_text.as_deref(), Some("hello world"));
            }
            _ => panic!("expected listen"),
        }
    }

    #[test]
    fn global_debug_on_subcommand() {
        let cli = Cli::try_parse_from(["cantrik", "--debug-config", "doctor"]).expect("parse");
        assert!(cli.global.debug_config);
        assert!(matches!(cli.cmd, Some(Command::Doctor)));
    }

    #[test]
    fn parse_health_flags() {
        let cli = Cli::try_parse_from([
            "cantrik",
            "health",
            "--soft",
            "--no-clippy",
            "--no-test",
            "--timeout-sec",
            "60",
            "--tree",
            "--outdated",
            "--coverage",
            "--deny",
            "--audit",
            "--sarif",
        ])
        .expect("parse");
        match cli.cmd.expect("cmd") {
            Command::Health {
                soft,
                no_clippy,
                no_test,
                timeout_sec,
                tree,
                outdated,
                coverage,
                deny,
                audit,
                sarif,
            } => {
                assert!(soft);
                assert!(no_clippy);
                assert!(no_test);
                assert_eq!(timeout_sec, 60);
                assert!(tree);
                assert!(outdated);
                assert!(coverage);
                assert!(deny);
                assert!(audit);
                assert!(sarif);
            }
            _ => panic!("expected health"),
        }
    }

    #[test]
    fn parse_exec_remote_trailing_argv() {
        let cli = Cli::try_parse_from(["cantrik", "exec", "--remote", "--", "uname", "-a"])
            .expect("parse");
        match cli.cmd.expect("cmd") {
            Command::Exec {
                approve,
                remote,
                argv,
            } => {
                assert!(!approve);
                assert!(remote);
                assert_eq!(argv, vec!["uname", "-a"]);
            }
            _ => panic!("expected exec"),
        }
    }

    #[test]
    fn parse_sync_src_and_approve() {
        let cli = Cli::try_parse_from(["cantrik", "sync", "--src", "./dist"]).expect("parse");
        match cli.cmd.expect("cmd") {
            Command::Sync { approve, src } => {
                assert!(!approve);
                assert!(src.ends_with("dist"));
            }
            _ => panic!("expected sync"),
        }
        let cli = Cli::try_parse_from(["cantrik", "sync", "--approve"]).expect("parse");
        match cli.cmd.expect("cmd") {
            Command::Sync { approve, src } => {
                assert!(approve);
                assert_eq!(src, std::path::PathBuf::from("."));
            }
            _ => panic!("expected sync"),
        }
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
    fn parse_cost_and_serve_mcp() {
        let cli = Cli::try_parse_from(["cantrik", "cost"]).expect("parse");
        match cli.cmd.expect("cmd") {
            Command::Cost { session_only } => assert!(!session_only),
            _ => panic!("expected cost"),
        }
        let cli = Cli::try_parse_from(["cantrik", "cost", "--session-only"]).expect("parse");
        assert!(matches!(
            cli.cmd,
            Some(Command::Cost { session_only: true })
        ));
        let cli = Cli::try_parse_from(["cantrik", "serve", "--mcp"]).expect("parse");
        assert!(matches!(cli.cmd, Some(Command::Serve { mcp: true })));
    }

    #[test]
    fn parse_lsp_subcommand() {
        let cli = Cli::try_parse_from(["cantrik", "lsp"]).expect("parse");
        assert!(matches!(cli.cmd, Some(Command::Lsp)));
    }

    #[test]
    fn parse_init_subcommand() {
        let cli = Cli::try_parse_from(["cantrik", "init"]).expect("parse");
        match &cli.cmd {
            Some(Command::Init { template, path }) => {
                assert_eq!(template, "generic");
                assert_eq!(path, &std::path::PathBuf::from("."));
            }
            _ => panic!("expected init"),
        }
        let cli = Cli::try_parse_from(["cantrik", "init", "--template", "rust-cli", "/tmp/x"])
            .expect("parse");
        match &cli.cmd {
            Some(Command::Init { template, path }) => {
                assert_eq!(template, "rust-cli");
                assert_eq!(path, &std::path::PathBuf::from("/tmp/x"));
            }
            _ => panic!("expected init"),
        }
    }

    #[test]
    fn parse_mcp_call() {
        let cli = Cli::try_parse_from([
            "cantrik",
            "mcp",
            "call",
            "github",
            "search_repositories",
            "--json",
            r#"{"q":"cantrik"}"#,
        ])
        .expect("parse");
        match cli.cmd.expect("cmd") {
            Command::Mcp {
                sub: McpCommand::Call { server, tool, json },
            } => {
                assert_eq!(server, "github");
                assert_eq!(tool, "search_repositories");
                assert!(json.contains("cantrik"));
            }
            _ => panic!("expected mcp call"),
        }
    }

    #[test]
    fn parse_sprint16_git_web() {
        use super::{PrCommand, WebCommand, WorkspaceCommand};
        use crate::commands::workspace_cmd::WorkspaceBranchCommand;
        let cli = Cli::try_parse_from(["cantrik", "workspace", "branch", "start", "my-task"])
            .expect("parse");
        match cli.cmd.expect("cmd") {
            Command::Workspace { sub } => match sub {
                WorkspaceCommand::Branch { sub: b } => match b {
                    WorkspaceBranchCommand::Start { slug, allow_dirty } => {
                        assert_eq!(slug, "my-task");
                        assert!(!allow_dirty);
                    }
                },
                _ => panic!("expected branch"),
            },
            _ => panic!("expected workspace"),
        }
        let cli = Cli::try_parse_from(["cantrik", "pr", "create", "--approve"]).expect("parse");
        match cli.cmd.expect("cmd") {
            Command::Pr {
                sub: PrCommand::Create { approve, .. },
            } => assert!(approve),
            _ => panic!("expected pr create"),
        }
        let cli =
            Cli::try_parse_from(["cantrik", "review", "--worktree", "--soft"]).expect("parse");
        match cli.cmd.expect("cmd") {
            Command::Review { worktree, soft } => {
                assert!(worktree);
                assert!(soft);
            }
            _ => panic!("expected review"),
        }
        let cli = Cli::try_parse_from([
            "cantrik",
            "fix",
            "https://github.com/a/b/issues/1",
            "--fetch",
            "--approve",
            "--run-agents",
        ])
        .expect("parse");
        match cli.cmd.expect("cmd") {
            Command::Fix {
                issue_url,
                fetch,
                approve,
                run_agents,
                run_experiment,
            } => {
                assert!(issue_url.contains("issues/1"));
                assert!(fetch);
                assert!(approve);
                assert!(run_agents);
                assert!(!run_experiment);
            }
            _ => panic!("expected fix"),
        }
        let cli =
            Cli::try_parse_from(["cantrik", "web", "search", "rust", "async"]).expect("parse");
        match cli.cmd.expect("cmd") {
            Command::Web {
                sub: WebCommand::Search { query, .. },
            } => assert_eq!(query, vec!["rust", "async"]),
            _ => panic!("expected web search"),
        }
    }

    #[test]
    fn parse_diff_handoff_replay() {
        let cli =
            Cli::try_parse_from(["cantrik", "diff", "--staged", "--conflicts"]).expect("parse");
        match cli.cmd.expect("cmd") {
            Command::Diff {
                staged,
                text_only,
                conflicts,
            } => {
                assert!(staged);
                assert!(!text_only);
                assert!(conflicts);
            }
            _ => panic!("expected diff"),
        }
        let cli =
            Cli::try_parse_from(["cantrik", "handoff", "--message", "ship it"]).expect("parse");
        match cli.cmd.expect("cmd") {
            Command::Handoff { message } => assert_eq!(message.as_deref(), Some("ship it")),
            _ => panic!("expected handoff"),
        }
        let cli =
            Cli::try_parse_from(["cantrik", "export", "-o", "/tmp/bundle.json"]).expect("parse");
        match cli.cmd.expect("cmd") {
            Command::Export { output } => {
                assert_eq!(output, std::path::Path::new("/tmp/bundle.json"))
            }
            _ => panic!("expected export"),
        }
        let cli = Cli::try_parse_from(["cantrik", "import", "-i", "ctx.json", "--seed-session"])
            .expect("parse");
        match cli.cmd.expect("cmd") {
            Command::Import {
                input,
                seed_session,
            } => {
                assert_eq!(input, std::path::Path::new("ctx.json"));
                assert!(seed_session);
            }
            _ => panic!("expected import"),
        }
        let cli =
            Cli::try_parse_from(["cantrik", "replay", "export", "-o", "r.json"]).expect("parse");
        match cli.cmd.expect("cmd") {
            Command::Replay { sub } => match sub {
                ReplayCommand::Export { output } => {
                    assert_eq!(output, std::path::Path::new("r.json"))
                }
                _ => panic!("expected replay export"),
            },
            _ => panic!("expected replay"),
        }
        let cli = Cli::try_parse_from(["cantrik", "replay", "play", "s.json"]).expect("parse");
        match cli.cmd.expect("cmd") {
            Command::Replay { sub } => match sub {
                ReplayCommand::Play { file } => assert_eq!(file, std::path::Path::new("s.json")),
                _ => panic!("expected replay play"),
            },
            _ => panic!("expected replay"),
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
