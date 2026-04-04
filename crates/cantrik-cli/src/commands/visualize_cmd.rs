//! `cantrik visualize` — Mermaid export (Sprint 18).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use cantrik_core::visualize::{self, VisualizeKind};

use crate::cli::VisualizeCliKind;

fn map_kind(k: VisualizeCliKind) -> VisualizeKind {
    match k {
        VisualizeCliKind::Callgraph => VisualizeKind::Callgraph,
        VisualizeCliKind::Architecture => VisualizeKind::Architecture,
        VisualizeCliKind::Dependencies => VisualizeKind::Dependencies,
    }
}

pub fn run(cwd: &Path, mode: VisualizeCliKind, output: Option<PathBuf>) -> ExitCode {
    let kind = map_kind(mode);
    let text = match visualize::render_visualize_kind(kind, cwd) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("visualize: {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Some(p) = output {
        if let Err(e) = fs::write(&p, &text) {
            eprintln!("visualize: write {}: {e}", p.display());
            return ExitCode::FAILURE;
        }
        eprintln!("visualize: wrote {}", p.display());
    } else {
        print!("{text}");
    }
    ExitCode::SUCCESS
}
