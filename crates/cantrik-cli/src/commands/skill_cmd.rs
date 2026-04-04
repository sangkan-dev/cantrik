//! Local skill registry install/list/remove (Sprint 13, PRD §7).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use cantrik_core::session::share_dir;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct Manifest {
    name: String,
    #[allow(dead_code)]
    version: String,
    files: Vec<String>,
}

/// TOML shape: `[<package_name>]` with `files = [...]`.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(transparent)]
struct InstalledRoot(BTreeMap<String, InstalledPkg>);

#[derive(Debug, Serialize, Deserialize, Clone)]
struct InstalledPkg {
    files: Vec<String>,
}

fn registry_root() -> PathBuf {
    share_dir().join("skill-registry")
}

fn installed_path(cwd: &Path) -> PathBuf {
    cwd.join(".cantrik").join("installed-skills.toml")
}

fn load_installed(cwd: &Path) -> InstalledRoot {
    let p = installed_path(cwd);
    if !p.is_file() {
        return InstalledRoot::default();
    }
    fs::read_to_string(&p)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_installed(cwd: &Path, root: &InstalledRoot) -> std::io::Result<()> {
    let p = installed_path(cwd);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }
    let s = toml::to_string_pretty(root)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    fs::write(p, s)
}

pub fn install(cwd: &Path, name: &str) -> ExitCode {
    let pkg = registry_root().join(name);
    let man_path = pkg.join("manifest.toml");
    if !man_path.is_file() {
        eprintln!(
            "cantrik skill install: no package '{}' at {}",
            name,
            man_path.display()
        );
        return ExitCode::from(1);
    }
    let raw = match fs::read_to_string(&man_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("cantrik skill install: read manifest: {e}");
            return ExitCode::FAILURE;
        }
    };
    let manifest: Manifest = match toml::from_str(&raw) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("cantrik skill install: manifest parse: {e}");
            return ExitCode::from(1);
        }
    };
    if manifest.name != name {
        eprintln!(
            "cantrik skill install: manifest name '{}' does not match package '{}'",
            manifest.name, name
        );
        return ExitCode::from(1);
    }

    let cantrik = cwd.join(".cantrik");
    let mut installed_files = Vec::new();
    for rel in &manifest.files {
        let from = pkg.join(rel);
        let to = cantrik.join(rel);
        if !from.is_file() {
            eprintln!(
                "cantrik skill install: missing file in package: {}",
                from.display()
            );
            return ExitCode::from(1);
        }
        if let Some(parent) = to.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Err(e) = fs::copy(&from, &to) {
            eprintln!("cantrik skill install: copy {}: {e}", rel);
            return ExitCode::FAILURE;
        }
        installed_files.push(rel.clone());
    }

    let mut root = load_installed(cwd);
    root.0.insert(
        name.to_string(),
        InstalledPkg {
            files: installed_files,
        },
    );
    if let Err(e) = save_installed(cwd, &root) {
        eprintln!("cantrik skill install: save state: {e}");
        return ExitCode::FAILURE;
    }

    println!("installed skill package '{name}' into .cantrik/");
    ExitCode::SUCCESS
}

pub fn list_registry() -> ExitCode {
    let root = registry_root();
    if !root.is_dir() {
        println!("(no local registry at {})", root.display());
        return ExitCode::SUCCESS;
    }
    let mut names: Vec<_> = fs::read_dir(&root)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            if p.is_dir() && p.join("manifest.toml").is_file() {
                Some(e.file_name().to_string_lossy().into_owned())
            } else {
                None
            }
        })
        .collect();
    names.sort();
    if names.is_empty() {
        println!("(empty registry at {})", root.display());
    } else {
        for n in names {
            println!("{n}");
        }
    }
    ExitCode::SUCCESS
}

pub fn remove(cwd: &Path, name: &str) -> ExitCode {
    let mut root = load_installed(cwd);
    let Some(pkg) = root.0.remove(name) else {
        eprintln!("cantrik skill remove: '{name}' is not recorded as installed here");
        return ExitCode::from(1);
    };
    for rel in &pkg.files {
        let p = cwd.join(".cantrik").join(rel);
        let _ = fs::remove_file(&p);
    }
    if let Err(e) = save_installed(cwd, &root) {
        eprintln!("cantrik skill remove: save state: {e}");
        return ExitCode::FAILURE;
    }
    println!("removed skill package '{name}'");
    ExitCode::SUCCESS
}

pub fn update(cwd: &Path, name: &str) -> ExitCode {
    let mut root = load_installed(cwd);
    if let Some(pkg) = root.0.remove(name) {
        for rel in &pkg.files {
            let _ = fs::remove_file(cwd.join(".cantrik").join(rel));
        }
        let _ = save_installed(cwd, &root);
    }
    install(cwd, name)
}
