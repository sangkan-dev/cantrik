use std::path::Path;
use std::process::ExitCode;

use cantrik_core::checkpoint::{
    apply_checkpoint, latest_checkpoint_dir, list_checkpoints, resolve_checkpoint_dir,
};

pub(crate) fn run(cwd: &Path, list: bool, id: Option<&str>) -> ExitCode {
    if list {
        match list_checkpoints(cwd) {
            Ok(v) => {
                if v.is_empty() {
                    println!("rollback: no checkpoints under .cantrik/checkpoints/");
                    return ExitCode::SUCCESS;
                }
                for (p, m) in &v {
                    let name = p.file_name().unwrap_or_default().to_string_lossy();
                    println!(
                        "{:03}  {}  {}  files={}",
                        m.seq,
                        m.created_at,
                        name,
                        m.files.len()
                    );
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("rollback --list: {e}");
                ExitCode::FAILURE
            }
        }
    } else {
        let dir = match id {
            Some(i) => match resolve_checkpoint_dir(cwd, i) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("rollback: {e}");
                    return ExitCode::FAILURE;
                }
            },
            None => match latest_checkpoint_dir(cwd) {
                Ok(Some(d)) => d,
                Ok(None) => {
                    eprintln!("rollback: no checkpoints; use `cantrik rollback --list`");
                    return ExitCode::FAILURE;
                }
                Err(e) => {
                    eprintln!("rollback: {e}");
                    return ExitCode::FAILURE;
                }
            },
        };
        match apply_checkpoint(cwd, &dir) {
            Ok(()) => {
                eprintln!("rollback: restored from {}", dir.display());
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("rollback: {e}");
                ExitCode::FAILURE
            }
        }
    }
}
