use std::path::Path;
use std::process::ExitCode;

use cantrik_core::config::AppConfig;
use cantrik_core::search::semantic_search;

pub(crate) async fn run(
    config: &AppConfig,
    project: Option<&Path>,
    query_words: &[String],
    limit: usize,
) -> ExitCode {
    let root = match project {
        Some(p) => p.to_path_buf(),
        None => match std::env::current_dir() {
            Ok(cwd) => cwd,
            Err(e) => {
                eprintln!("failed to get current directory: {e}");
                return ExitCode::FAILURE;
            }
        },
    };

    let root = match std::fs::canonicalize(&root) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("failed to resolve search root {}: {e}", root.display());
            return ExitCode::FAILURE;
        }
    };

    let query = query_words.join(" ");
    if query.trim().is_empty() {
        eprintln!("search: query is empty");
        return ExitCode::from(2);
    }

    let top_k = limit.max(1);
    match semantic_search(&root, config, &query, top_k).await {
        Ok(rows) => {
            for r in rows {
                println!(
                    "{:.4}\t{}\t{}\t{}\n{}",
                    r.score, r.path, r.symbol, r.language, r.preview
                );
                println!("---");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("search failed: {e}");
            ExitCode::FAILURE
        }
    }
}
