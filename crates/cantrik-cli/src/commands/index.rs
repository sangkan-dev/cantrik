use std::path::Path;
use std::process::ExitCode;

use cantrik_core::config::AppConfig;
use cantrik_core::indexing::{self, IndexOptions};
use cantrik_core::search::build_vector_index;

pub(crate) async fn run(config: &AppConfig, path: Option<&Path>, no_vectors: bool) -> ExitCode {
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

    let root_clone = root.clone();
    let opts = IndexOptions::default();
    let report = match tokio::task::spawn_blocking(move || {
        indexing::build_index(&root_clone, &opts)
    })
    .await
    {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => {
            eprintln!("index failed: {e}");
            return ExitCode::FAILURE;
        }
        Err(e) => {
            eprintln!("index task failed: {e}");
            return ExitCode::FAILURE;
        }
    };

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

    if no_vectors {
        println!("  vectors:                 skipped (--no-vectors)");
        return ExitCode::SUCCESS;
    }

    match build_vector_index(&root, config).await {
        Ok(v) => {
            println!(
                "  vectors (LanceDB):       {} chunks, embedded {}, reused {}",
                v.chunks_total, v.chunks_embedded, v.chunks_reused
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("vector index failed: {e}");
            ExitCode::FAILURE
        }
    }
}
