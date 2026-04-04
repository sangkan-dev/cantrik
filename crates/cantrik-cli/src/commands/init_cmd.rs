//! `cantrik init` — bootstrap `.cantrik/` from built-in templates (Sprint 19).

use std::fs;
use std::path::Path;
use std::process::ExitCode;

const GENERIC_CANTRIK: &str = include_str!("../../templates/init/generic/cantrik.toml");
const GENERIC_RULES: &str = include_str!("../../templates/init/generic/rules.md");
const RUST_CANTRIK: &str = include_str!("../../templates/init/rust-cli/cantrik.toml");
const RUST_RULES: &str = include_str!("../../templates/init/rust-cli/rules.md");

fn template_files(name: &str) -> Result<(&'static str, &'static str), String> {
    match name {
        "generic" => Ok((GENERIC_CANTRIK, GENERIC_RULES)),
        "rust-cli" => Ok((RUST_CANTRIK, RUST_RULES)),
        _ => Err(format!(
            "unknown template {name:?}; use generic or rust-cli"
        )),
    }
}

/// Writes `.cantrik/cantrik.toml` and `.cantrik/rules.md` when missing.
pub fn run(project_root: &Path, template: &str) -> ExitCode {
    let (cantrik_body, rules_body) = match template_files(template) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("cantrik init: {e}");
            return ExitCode::from(2);
        }
    };

    let dot = project_root.join(".cantrik");
    if let Err(e) = fs::create_dir_all(&dot) {
        eprintln!("cantrik init: failed to create {}: {e}", dot.display());
        return ExitCode::FAILURE;
    }

    let cantrik_path = dot.join("cantrik.toml");
    if cantrik_path.exists() {
        eprintln!(
            "cantrik init: {} already exists; not overwriting",
            cantrik_path.display()
        );
        return ExitCode::from(2);
    }
    if let Err(e) = fs::write(&cantrik_path, cantrik_body) {
        eprintln!("cantrik init: write {}: {e}", cantrik_path.display());
        return ExitCode::FAILURE;
    }

    let rules_path = dot.join("rules.md");
    if rules_path.exists() {
        eprintln!(
            "cantrik init: {} already exists; skipped rules.md",
            rules_path.display()
        );
        println!(
            "Initialized {} (template {template})",
            cantrik_path.display()
        );
        return ExitCode::SUCCESS;
    }
    if let Err(e) = fs::write(&rules_path, rules_body) {
        eprintln!("cantrik init: write {}: {e}", rules_path.display());
        return ExitCode::FAILURE;
    }

    println!(
        "Initialized {} and {} (template {template})",
        cantrik_path.display(),
        rules_path.display()
    );
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{run, template_files};

    #[test]
    fn template_names() {
        assert!(template_files("generic").is_ok());
        assert!(template_files("rust-cli").is_ok());
        assert!(template_files("nope").is_err());
    }

    #[test]
    fn init_writes_dot_cantrik() {
        let base = std::env::temp_dir().join(format!("cantrik-init-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).expect("mkdir");
        let code = run(&base, "generic");
        assert_eq!(code, std::process::ExitCode::SUCCESS);
        assert!(base.join(".cantrik/cantrik.toml").is_file());
        assert!(base.join(".cantrik/rules.md").is_file());
        let _ = fs::remove_dir_all(&base);
    }
}
