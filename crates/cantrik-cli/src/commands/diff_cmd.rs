use std::path::Path;
use std::process::ExitCode;

use cantrik_core::config::{effective_collab_max_files_in_report, load_merged_config};
use cantrik_core::semantic_diff::{build_semantic_report, list_conflicts};

pub(crate) async fn run(
    cwd: &Path,
    staged: bool,
    text_only: bool,
    show_conflicts: bool,
) -> ExitCode {
    let config = match load_merged_config(cwd) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("failed to load config: {e}");
            return ExitCode::FAILURE;
        }
    };
    let max_files = Some(effective_collab_max_files_in_report(&config.collab));
    let include_semantic = !text_only;

    let report = match build_semantic_report(cwd, staged, include_semantic, max_files) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("cantrik diff: {e}");
            return ExitCode::FAILURE;
        }
    };

    if include_semantic && !report.index_present {
        eprintln!(
            "note: AST index missing or empty (`.cantrik/index/ast/`). Run `cantrik index` for symbol/caller overlay."
        );
    }

    println!(
        "risk: {:?}  files_changed: {}  approx_loc_added(+): {}",
        report.risk,
        report.files.len(),
        report.approx_loc_added
    );
    for n in &report.risk_notes {
        println!("  risk note: {n}");
    }
    if !report.test_hints.is_empty() {
        println!("test hints:");
        for h in &report.test_hints {
            println!("  - {h}");
        }
    }

    for f in &report.files {
        println!("--- {}", f.path);
        if !f.affected_symbols.is_empty() {
            println!("affected symbols: {}", f.affected_symbols.join(", "));
        }
        if !f.callers.is_empty() {
            println!("callers (intra-file):");
            for c in &f.callers {
                println!("  {c}");
            }
        }
        print!("{}", f.unified_diff);
    }

    if show_conflicts {
        println!("--- git conflicts (if any) ---");
        match list_conflicts(cwd) {
            Ok((entries, hints)) => {
                if entries.is_empty() {
                    println!("(no unmerged paths in git status --porcelain)");
                } else {
                    for e in &entries {
                        println!("  {} {}", e.xy, e.path);
                    }
                    for h in hints {
                        println!("{h}");
                    }
                }
            }
            Err(e) => eprintln!("conflict check: {e}"),
        }
    }

    ExitCode::SUCCESS
}
