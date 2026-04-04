use std::fs;
use std::path::Path;
use std::process::ExitCode;

use cantrik_core::config::{effective_collab_replay_tail_messages, load_merged_config};
use cantrik_core::semantic_diff::{
    export_context_bundle, export_session_replay_json, format_replay_timeline,
    import_context_bundle, parse_session_replay, serialize_context_bundle,
    serialize_session_replay, write_handoff_markdown,
};
use cantrik_core::session::connect_pool;

pub(crate) async fn handoff(cwd: &Path, message: Option<String>) -> ExitCode {
    let pool = match connect_pool().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("handoff: cannot open session DB: {e}");
            return ExitCode::FAILURE;
        }
    };
    match write_handoff_markdown(&pool, cwd, message.as_deref()).await {
        Ok(path) => {
            println!("{}", path.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("handoff: {e}");
            ExitCode::FAILURE
        }
    }
}

pub(crate) async fn export_context(cwd: &Path, output: &Path) -> ExitCode {
    let config = match load_merged_config(cwd) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("failed to load config: {e}");
            return ExitCode::FAILURE;
        }
    };
    let tail = effective_collab_replay_tail_messages(&config.collab);
    let pool = match connect_pool().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("export: cannot open session DB: {e}");
            return ExitCode::FAILURE;
        }
    };
    match export_context_bundle(&pool, cwd, tail).await {
        Ok(bundle) => match serialize_context_bundle(&bundle) {
            Ok(json) => match fs::write(output, json) {
                Ok(()) => {
                    println!("{}", output.display());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("export: write {}: {e}", output.display());
                    ExitCode::FAILURE
                }
            },
            Err(e) => {
                eprintln!("export: serialize: {e}");
                ExitCode::FAILURE
            }
        },
        Err(e) => {
            eprintln!("export: {e}");
            ExitCode::FAILURE
        }
    }
}

pub(crate) async fn import_context(cwd: &Path, input: &Path, seed_session: bool) -> ExitCode {
    let json = match fs::read_to_string(input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("import: read {}: {e}", input.display());
            return ExitCode::FAILURE;
        }
    };
    let pool = match connect_pool().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("import: cannot open session DB: {e}");
            return ExitCode::FAILURE;
        }
    };
    match import_context_bundle(&pool, cwd, &json, seed_session).await {
        Ok(()) => {
            println!("imported into {}", cwd.join(".cantrik").display());
            if seed_session {
                println!("(appended one assistant message with imported transcript summary)");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("import: {e}");
            ExitCode::FAILURE
        }
    }
}

pub(crate) async fn replay_export(cwd: &Path, output: &Path) -> ExitCode {
    let config = match load_merged_config(cwd) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("failed to load config: {e}");
            return ExitCode::FAILURE;
        }
    };
    let tail = effective_collab_replay_tail_messages(&config.collab);
    let pool = match connect_pool().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("replay export: cannot open session DB: {e}");
            return ExitCode::FAILURE;
        }
    };
    match export_session_replay_json(&pool, cwd, tail).await {
        Ok(file) => match serialize_session_replay(&file) {
            Ok(json) => match fs::write(output, json) {
                Ok(()) => {
                    println!("{}", output.display());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("replay export: write {}: {e}", output.display());
                    ExitCode::FAILURE
                }
            },
            Err(e) => {
                eprintln!("replay export: serialize: {e}");
                ExitCode::FAILURE
            }
        },
        Err(e) => {
            eprintln!("replay export: {e}");
            ExitCode::FAILURE
        }
    }
}

pub(crate) fn replay_play(path: &Path) -> ExitCode {
    let json = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("replay play: read {}: {e}", path.display());
            return ExitCode::FAILURE;
        }
    };
    match parse_session_replay(&json) {
        Ok(file) => {
            print!("{}", format_replay_timeline(&file));
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("replay play: {e}");
            ExitCode::FAILURE
        }
    }
}
