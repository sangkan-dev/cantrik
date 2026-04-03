use std::io::{self, Read};
use std::path::Path;
use std::process::ExitCode;

use cantrik_core::config::AppConfig;
use cantrik_core::tool_system::{tool_read_file, tool_write_file};
use cantrik_core::tools::{WriteApproval, diff_for_new_contents};

const STDIN_MAX: u64 = 4 * 1024 * 1024;

fn max_read_bytes(config: &AppConfig) -> u64 {
    config.memory.max_file_read_bytes.unwrap_or(STDIN_MAX)
}

pub(crate) fn read_run(config: &AppConfig, project_root: &Path, path: &Path) -> ExitCode {
    let max = max_read_bytes(config);
    match tool_read_file(config, project_root, path, max) {
        Ok(s) => {
            print!("{s}");
            if !s.ends_with('\n') {
                println!();
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("file read: {e}");
            ExitCode::FAILURE
        }
    }
}

fn read_new_content(content_file: Option<&Path>, max_bytes: u64) -> Result<String, io::Error> {
    if let Some(p) = content_file {
        std::fs::read_to_string(p)
    } else {
        let mut buf = Vec::new();
        io::stdin()
            .take(max_bytes.saturating_add(1))
            .read_to_end(&mut buf)?;
        if buf.len() as u64 > max_bytes {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "stdin larger than limit",
            ));
        }
        String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

pub(crate) fn write_run(
    config: &AppConfig,
    project_root: &Path,
    path: &Path,
    content_file: Option<&Path>,
    approve: bool,
) -> ExitCode {
    let max_in = max_read_bytes(config);
    let new_text = match read_new_content(content_file, max_in) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("file write: {e}");
            return ExitCode::FAILURE;
        }
    };

    let diff = match diff_for_new_contents(path, &new_text) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("file write: {e}");
            return ExitCode::FAILURE;
        }
    };

    print!("{diff}");
    if !diff.ends_with('\n') {
        println!();
    }

    if !approve {
        eprintln!("file write: no changes written (use --approve after reviewing diff above)");
        return ExitCode::SUCCESS;
    }

    match tool_write_file(
        config,
        project_root,
        path,
        &new_text,
        WriteApproval::user_confirmed_after_reviewing_diff(),
    ) {
        Ok(()) => {
            eprintln!("file write: wrote {}", path.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("file write: {e}");
            ExitCode::FAILURE
        }
    }
}
