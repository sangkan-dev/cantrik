use std::path::Path;
use std::process::ExitCode;

use cantrik_core::config::AppConfig;
use cantrik_core::indexing::{self, IndexOptions};

pub(crate) fn run(_config: &AppConfig, path: Option<&Path>) -> ExitCode {
    let root = match path {
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
            eprintln!("failed to resolve index root {}: {e}", root.display());
            return ExitCode::FAILURE;
        }
    };

    let opts = IndexOptions::default();
    match indexing::build_index(&root, &opts) {
        Ok(report) => {
            let ast = indexing::ast_index_dir(&root);
            println!("index: project root {}", root.display());
            println!("  files scanned:           {}", report.files_scanned);
            println!("  files indexed (parsed):  {}", report.files_indexed);
            println!("  files reused (unchanged):{}", report.files_reused);
            println!("  skipped (size limit):    {}", report.files_skipped_size);
            println!("  skipped (binary):        {}", report.files_skipped_binary);
            println!(
                "  skipped (unsupported):   {}",
                report.files_skipped_unsupported
            );
            println!("  chunks:                  {}", report.chunks);
            println!("  call edges (intra-file): {}", report.edges);
            println!("  artifacts:               {}", ast.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("index failed: {e}");
            ExitCode::FAILURE
        }
    }
}
