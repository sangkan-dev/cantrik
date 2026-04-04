//! Macro record/replay (Sprint 13, PRD §4.18).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Recording {
    label: String,
    steps: Vec<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MacroFile {
    label: String,
    steps: Vec<Vec<String>>,
}

fn macros_dir(cwd: &Path) -> PathBuf {
    cwd.join(".cantrik").join("macros")
}

fn recording_path(cwd: &Path) -> PathBuf {
    macros_dir(cwd).join(".recording.json")
}

fn slug(label: &str) -> String {
    label
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn macro_file_path(cwd: &Path, label: &str) -> PathBuf {
    macros_dir(cwd).join(format!("{}.json", slug(label)))
}

pub fn record(cwd: &Path, label: &str) -> ExitCode {
    let dir = macros_dir(cwd);
    if let Err(e) = fs::create_dir_all(&dir) {
        eprintln!("cantrik macro record: {e}");
        return ExitCode::FAILURE;
    }
    let rp = recording_path(cwd);
    if rp.is_file() {
        eprintln!("cantrik macro record: already recording; run `cantrik macro stop` first");
        return ExitCode::from(1);
    }
    let rec = Recording {
        label: label.to_string(),
        steps: Vec::new(),
    };
    let json = match serde_json::to_string_pretty(&rec) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("cantrik macro record: {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = fs::write(&rp, json) {
        eprintln!("cantrik macro record: {e}");
        return ExitCode::FAILURE;
    }
    println!(
        "recording macro {:?} — use `cantrik macro add -- <argv...>` then `cantrik macro stop`",
        label
    );
    ExitCode::SUCCESS
}

pub fn add(cwd: &Path, args: &[String]) -> ExitCode {
    let rp = recording_path(cwd);
    if !rp.is_file() {
        eprintln!("cantrik macro add: not recording; start with `cantrik macro record \"label\"`");
        return ExitCode::from(1);
    }
    let raw = match fs::read_to_string(&rp) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("cantrik macro add: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut rec: Recording = match serde_json::from_str(&raw) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("cantrik macro add: {e}");
            return ExitCode::from(1);
        }
    };
    if args.is_empty() {
        eprintln!("cantrik macro add: need at least one argument");
        return ExitCode::from(2);
    }
    rec.steps.push(args.to_vec());
    let json = match serde_json::to_string_pretty(&rec) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("cantrik macro add: {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = fs::write(&rp, json) {
        eprintln!("cantrik macro add: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

pub fn stop(cwd: &Path) -> ExitCode {
    let rp = recording_path(cwd);
    if !rp.is_file() {
        eprintln!("cantrik macro stop: no active recording");
        return ExitCode::from(1);
    }
    let raw = match fs::read_to_string(&rp) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("cantrik macro stop: {e}");
            return ExitCode::FAILURE;
        }
    };
    let rec: Recording = match serde_json::from_str(&raw) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("cantrik macro stop: {e}");
            return ExitCode::from(1);
        }
    };
    if rec.steps.is_empty() {
        eprintln!("cantrik macro stop: no steps recorded; discarding");
        let _ = fs::remove_file(&rp);
        return ExitCode::from(1);
    }
    let out = MacroFile {
        label: rec.label.clone(),
        steps: rec.steps,
    };
    let path = macro_file_path(cwd, &rec.label);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let json = match serde_json::to_string_pretty(&out) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("cantrik macro stop: {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = fs::write(&path, json) {
        eprintln!("cantrik macro stop: {e}");
        return ExitCode::FAILURE;
    }
    let _ = fs::remove_file(&rp);
    println!("saved macro to {}", path.display());
    ExitCode::SUCCESS
}

pub fn run_macro(cwd: &Path, label: &str) -> ExitCode {
    let path = macro_file_path(cwd, label);
    if !path.is_file() {
        eprintln!("cantrik macro run: no macro file for {:?}", label);
        return ExitCode::from(1);
    }
    let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("cantrik macro run: {e}");
            return ExitCode::FAILURE;
        }
    };
    let m: MacroFile = match serde_json::from_str(&raw) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("cantrik macro run: {e}");
            return ExitCode::from(1);
        }
    };
    for (i, step) in m.steps.iter().enumerate() {
        if step.is_empty() {
            eprintln!("cantrik macro run: step {i} is empty");
            return ExitCode::from(1);
        }
        let prog = &step[0];
        let status = Command::new(prog)
            .args(&step[1..])
            .current_dir(cwd)
            .status();
        match status {
            Ok(s) if s.success() => {}
            Ok(s) => {
                eprintln!("cantrik macro run: step {i} {:?} exited with {s}", step);
                let code = s.code().unwrap_or(1);
                return ExitCode::from((code & 0xFF) as u8);
            }
            Err(e) => {
                eprintln!("cantrik macro run: step {i}: {e}");
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}

pub fn list_macros(cwd: &Path) -> ExitCode {
    let dir = macros_dir(cwd);
    if !dir.is_dir() {
        println!("(no macros)");
        return ExitCode::SUCCESS;
    }
    let mut names = Vec::new();
    for e in fs::read_dir(&dir).into_iter().flatten().flatten() {
        let p = e.path();
        if p.is_file() {
            let n = e.file_name().to_string_lossy().into_owned();
            if n.ends_with(".json") && n != ".recording.json" {
                names.push(n.trim_end_matches(".json").to_string());
            }
        }
    }
    names.sort();
    if names.is_empty() {
        println!("(no saved macros)");
    } else {
        for n in names {
            println!("{n}");
        }
    }
    ExitCode::SUCCESS
}
