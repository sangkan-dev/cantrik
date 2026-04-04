//! Sprint 17: explain, teach, why, upgrade, audit.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use cantrik_core::config::{
    AppConfig, effective_explain_max_blame_lines, effective_teach_max_files_scanned,
};
use cantrik_core::intelligence::{
    IntelligenceError, apply_wiki_format, build_explain_why_prompt, build_teach_prompt,
    build_upgrade_suggestion_prompt, build_why_context, collect_explain_context,
    gather_teach_context, read_cargo_lock_excerpt, run_cargo_audit, run_cargo_tree_depth,
    run_cargo_tree_invert,
};
use cantrik_core::llm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeachFormat {
    Markdown,
    Wiki,
}

pub async fn run_explain(
    config: &AppConfig,
    cwd: &Path,
    path: &PathBuf,
    line: Option<u32>,
    why: bool,
) -> ExitCode {
    let meta = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("explain: {e}");
            return ExitCode::FAILURE;
        }
    };
    if !meta.is_file() {
        eprintln!("explain: path must be a file (got {})", path.display());
        return ExitCode::from(2);
    }
    let max_blame = effective_explain_max_blame_lines(&config.intelligence);
    let ctx = match collect_explain_context(cwd, path, line, max_blame, 20) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("explain: {e}");
            return ExitCode::FAILURE;
        }
    };
    if !why {
        println!("{ctx}");
        return ExitCode::SUCCESS;
    }
    let rel_display = path.display().to_string();
    let prompt = build_explain_why_prompt(&ctx, &rel_display);
    match llm::ask_complete_text(config, &prompt).await {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("explain: LLM error: {e}");
            ExitCode::FAILURE
        }
    }
}

pub async fn run_teach(
    config: &AppConfig,
    cwd: &Path,
    output_dir: Option<&Path>,
    format: TeachFormat,
) -> ExitCode {
    let max_files = effective_teach_max_files_scanned(&config.intelligence);
    let ctx = match gather_teach_context(cwd, max_files) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("teach: {e}");
            return ExitCode::FAILURE;
        }
    };
    let wiki = format == TeachFormat::Wiki;
    let prompt = build_teach_prompt(&ctx, wiki);
    let raw = match llm::ask_complete_text(config, &prompt).await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("teach: LLM error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let body = if wiki {
        apply_wiki_format("Architecture", &raw)
    } else {
        raw
    };
    if let Some(dir) = output_dir {
        if let Err(e) = fs::create_dir_all(dir) {
            eprintln!("teach: mkdir {}: {e}", dir.display());
            return ExitCode::FAILURE;
        }
        let path = dir.join("ARCHITECTURE.md");
        if let Err(e) = fs::write(&path, &body) {
            eprintln!("teach: write {}: {e}", path.display());
            return ExitCode::FAILURE;
        }
        eprintln!("teach: wrote {}", path.display());
    } else {
        println!("{body}");
    }
    ExitCode::SUCCESS
}

pub async fn run_why(
    config: &AppConfig,
    cwd: &Path,
    crate_name: &str,
    synthesize: bool,
) -> ExitCode {
    let tree = match run_cargo_tree_invert(cwd, crate_name) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("why: {e}");
            return ExitCode::from(2);
        }
    };
    let max_m = effective_teach_max_files_scanned(&config.intelligence).max(32);
    let block = build_why_context(cwd, crate_name, &tree, max_m);
    if !synthesize {
        println!("{block}");
        return ExitCode::SUCCESS;
    }
    let prompt = format!(
        "Summarize why `{crate}` appears in this Rust dependency tree and what to check next. Use the facts only.\n\n{block}",
        crate = crate_name,
        block = block
    );
    match llm::ask_complete_text(config, &prompt).await {
        Ok(t) => {
            println!("{t}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("why: LLM error: {e}");
            ExitCode::FAILURE
        }
    }
}

pub async fn run_upgrade(config: &AppConfig, cwd: &Path) -> ExitCode {
    let lock = match read_cargo_lock_excerpt(cwd, 24 * 1024) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("upgrade: {e}");
            return ExitCode::FAILURE;
        }
    };
    let tree = match run_cargo_tree_depth(cwd, 1) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("upgrade: {e}");
            return ExitCode::FAILURE;
        }
    };
    let tree_excerpt: String = tree.lines().take(120).collect::<Vec<_>>().join("\n");
    let prompt = build_upgrade_suggestion_prompt(&lock, &tree_excerpt);
    match llm::ask_complete_text(config, &prompt).await {
        Ok(t) => {
            println!("{t}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("upgrade: LLM error: {e}");
            ExitCode::FAILURE
        }
    }
}

pub fn run_audit(config: &AppConfig, cwd: &Path) -> ExitCode {
    match run_cargo_audit(cwd, &config.intelligence) {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            if !stdout.is_empty() {
                print!("{stdout}");
            }
            if !stderr.is_empty() {
                eprint!("{stderr}");
            }
            if out.status.success() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(IntelligenceError::Cargo(msg))
            if msg.contains("failed to run") || msg.contains("No such file") =>
        {
            eprintln!(
                "audit: could not run audit command. Install cargo-audit (`cargo install cargo-audit`) or set [intelligence].audit_command in config.\nDetails: {msg}"
            );
            ExitCode::from(2)
        }
        Err(e) => {
            eprintln!("audit: {e}");
            ExitCode::FAILURE
        }
    }
}
