//! Static extension catalog (`extensions.json`) — list/show only; no network fetch in MVP.

use std::borrow::Cow;
use std::path::Path;
use std::process::ExitCode;

use serde::Deserialize;

use crate::cli::RegistryCommand;

/// Mirrors `apps/cantrik-site/static/registry/extensions.json` in the repo at compile time.
const EMBEDDED_EXTENSIONS_JSON: &str =
    include_str!("../../../../apps/cantrik-site/static/registry/extensions.json");

#[derive(Debug, Deserialize)]
struct ExtensionCatalog {
    #[allow(dead_code)]
    schema_version: u32,
    extensions: Vec<ExtensionEntry>,
}

#[derive(Debug, Deserialize)]
struct ExtensionEntry {
    id: String,
    name: String,
    description: String,
    kind: String,
    source: String,
    install_hint: String,
    #[serde(default)]
    verified: Option<bool>,
    #[serde(default)]
    recipe_id: Option<String>,
}

fn load_catalog_json(file: Option<&Path>) -> Result<ExtensionCatalog, String> {
    let text: Cow<'_, str> = match file {
        None => Cow::Borrowed(EMBEDDED_EXTENSIONS_JSON),
        Some(path) => {
            let bytes = std::fs::read(path)
                .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
            let s = String::from_utf8(bytes)
                .map_err(|e| format!("{}: invalid UTF-8: {e}", path.display()))?;
            Cow::Owned(s)
        }
    };
    serde_json::from_str(&text).map_err(|e| format!("invalid extensions.json: {e}"))
}

/// Run `cantrik registry …`.
pub fn run(cmd: &RegistryCommand) -> ExitCode {
    match cmd {
        RegistryCommand::List { file } => match load_catalog_json(file.as_deref()) {
            Ok(cat) => {
                println!("id\tkind\tname");
                for e in &cat.extensions {
                    println!("{}\t{}\t{}", e.id, e.kind, e.name);
                }
                ExitCode::SUCCESS
            }
            Err(msg) => {
                eprintln!("cantrik registry list: {msg}");
                ExitCode::FAILURE
            }
        },
        RegistryCommand::Show { id, file } => match load_catalog_json(file.as_deref()) {
            Ok(cat) => {
                let id_trim = id.trim();
                let found = cat.extensions.iter().find(|e| e.id == id_trim);
                match found {
                    None => {
                        eprintln!("cantrik registry show: no extension with id {id_trim:?}");
                        ExitCode::from(2)
                    }
                    Some(e) => {
                        println!("id: {}", e.id);
                        println!("name: {}", e.name);
                        println!("kind: {}", e.kind);
                        if let Some(v) = e.verified {
                            println!("verified: {v}");
                        }
                        if let Some(r) = &e.recipe_id {
                            println!("recipe_id: {r}");
                        }
                        println!("description:\n{}", e.description);
                        println!("source:\n{}", e.source);
                        println!("install_hint:\n{}", e.install_hint);
                        ExitCode::SUCCESS
                    }
                }
            }
            Err(msg) => {
                eprintln!("cantrik registry show: {msg}");
                ExitCode::FAILURE
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_catalog_parses() {
        let cat: ExtensionCatalog =
            serde_json::from_str(EMBEDDED_EXTENSIONS_JSON).expect("embedded JSON");
        assert_eq!(cat.schema_version, 1);
        assert!(!cat.extensions.is_empty());
        assert!(cat.extensions.iter().any(|e| e.id == "example-skill-pack"));
    }
}
