//! Interactive `cantrik configure` — edit global or project TOML with `toml_edit` (preserves comments).

use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use cantrik_core::config::{AppConfig, resolve_config_paths};
use cantrik_core::llm::providers::ProviderKind;
use toml_edit::{DocumentMut, Item, Table};

fn read_line_trimmed() -> Option<String> {
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).ok()?;
    Some(buf.trim().to_string())
}

fn load_or_empty(path: &Path) -> Result<DocumentMut, String> {
    if !path.exists() {
        return Ok(DocumentMut::new());
    }
    let s = std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    s.parse::<DocumentMut>()
        .map_err(|e| format!("parse TOML {}: {e}", path.display()))
}

fn ensure_table<'a>(doc: &'a mut DocumentMut, key: &str) -> &'a mut Table {
    let need_new = match doc.get(key) {
        None => true,
        Some(i) => !i.is_table(),
    };
    if need_new {
        doc.insert(key, Item::Table(Table::new()));
    }
    doc.get_mut(key)
        .and_then(|i| i.as_table_mut())
        .expect("table section")
}

fn set_string(doc: &mut DocumentMut, section: &str, key: &str, value: &str) {
    let t = ensure_table(doc, section);
    t.insert(key, toml_edit::value(value));
}

fn set_bool(doc: &mut DocumentMut, section: &str, key: &str, value: bool) {
    let t = ensure_table(doc, section);
    t.insert(key, toml_edit::value(value));
}

fn remove_key(doc: &mut DocumentMut, section: &str, key: &str) {
    let Some(t) = doc.get_mut(section).and_then(|i| i.as_table_mut()) else {
        return;
    };
    t.remove(key);
}

fn write_with_backup(path: &Path, doc: &DocumentMut) -> Result<(), String> {
    let text = doc.to_string();
    toml::from_str::<AppConfig>(&text)
        .map_err(|e| format!("resulting TOML would not parse as AppConfig: {e}"))?;

    if path.exists() {
        let bak = backup_path(path);
        std::fs::copy(path, &bak).map_err(|e| format!("backup {}: {e}", bak.display()))?;
    } else if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }

    std::fs::write(path, &text).map_err(|e| format!("write {}: {e}", path.display()))?;
    Ok(())
}

fn backup_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "config.toml".to_string());
    path.with_file_name(format!("{name}.bak"))
}

fn print_env_hints() {
    println!(
        "\nAPI keys: set environment variables or use ${{VAR}} placeholders in ~/.config/cantrik/providers.toml"
    );
    println!("  Anthropic: ANTHROPIC_API_KEY");
    println!("  Google Gemini: GOOGLE_API_KEY or GEMINI_API_KEY (see providers loader)");
    println!("  OpenAI: OPENAI_API_KEY");
    println!("  OpenRouter: OPENROUTER_API_KEY");
    println!("  Groq: GROQ_API_KEY");
    println!("  Ollama: usually no key; base URL in [index].ollama_base or providers.toml");
    println!(
        "  providers file: {}",
        cantrik_core::llm::providers::providers_toml_path().display()
    );
}

fn pick_target_path(cwd: &Path, global: bool, project: bool) -> Result<PathBuf, String> {
    let paths = resolve_config_paths(cwd);
    if global {
        return Ok(paths.global);
    }
    if project {
        return Ok(paths.project);
    }

    if !io::stdin().is_terminal() {
        return Ok(paths.project);
    }

    println!("Edit which config?");
    println!("  1) Project: {}", paths.project.display());
    println!("  2) Global:  {}", paths.global.display());
    print!("Choice [1]: ");
    let _ = io::stdout().flush();
    let line = read_line_trimmed().unwrap_or_default();
    if line == "2" {
        Ok(paths.global)
    } else {
        Ok(paths.project)
    }
}

fn menu_llm(doc: &mut DocumentMut) -> Result<(), String> {
    println!("\n--- LLM provider ---");
    for (i, k) in ProviderKind::ALL.iter().enumerate() {
        println!("  {}) {}", i + 1, k.as_str());
    }
    print!("Select provider number (or Enter to skip): ");
    let _ = io::stdout().flush();
    let line = read_line_trimmed().unwrap_or_default();
    if line.is_empty() {
        return Ok(());
    }
    let n: usize = line.parse().map_err(|_| "invalid number")?;
    if n == 0 || n > ProviderKind::ALL.len() {
        return Err("out of range".into());
    }
    let kind = &ProviderKind::ALL[n - 1];
    set_string(doc, "llm", "provider", kind.as_str());

    print!("Model name (or Enter to leave unset / remove key): ");
    let _ = io::stdout().flush();
    let m = read_line_trimmed().unwrap_or_default();
    if m.is_empty() {
        remove_key(doc, "llm", "model");
    } else {
        set_string(doc, "llm", "model", &m);
    }
    Ok(())
}

fn menu_offline(doc: &mut DocumentMut) -> Result<(), String> {
    println!("\n--- Offline / air-gapped LLM ---");
    println!("  1) Enable  ([llm].offline = true)");
    println!("  2) Disable (remove [llm].offline)");
    print!("Choice (or Enter to skip): ");
    let _ = io::stdout().flush();
    let line = read_line_trimmed().unwrap_or_default();
    match line.as_str() {
        "1" => set_bool(doc, "llm", "offline", true),
        "2" => remove_key(doc, "llm", "offline"),
        _ => {}
    }
    Ok(())
}

fn menu_ollama_base(doc: &mut DocumentMut) -> Result<(), String> {
    println!("\n--- Ollama HTTP base ([index].ollama_base) ---");
    print!("URL (e.g. http://127.0.0.1:11434) or Enter to skip: ");
    let _ = io::stdout().flush();
    let line = read_line_trimmed().unwrap_or_default();
    if line.is_empty() {
        return Ok(());
    }
    set_string(doc, "index", "ollama_base", &line);
    Ok(())
}

pub fn run(cwd: &Path, global: bool, project: bool) -> ExitCode {
    let path = match pick_target_path(cwd, global, project) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("cantrik configure: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut doc = match load_or_empty(&path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("cantrik configure: {e}");
            return ExitCode::FAILURE;
        }
    };

    println!("Editing: {}", path.display());
    println!("(Changes are kept in memory until you choose Save.)");

    loop {
        println!("\n--- cantrik configure ---");
        println!("  1) LLM provider + model");
        println!("  2) Offline mode ([llm].offline)");
        println!("  3) Ollama base URL ([index].ollama_base)");
        println!("  4) Show API key / providers.toml hints");
        println!("  5) Save and exit");
        println!("  6) Quit without saving");
        print!("Choice: ");
        let _ = io::stdout().flush();
        let Some(line) = read_line_trimmed() else {
            break;
        };
        let err = match line.as_str() {
            "1" => menu_llm(&mut doc).err(),
            "2" => menu_offline(&mut doc).err(),
            "3" => menu_ollama_base(&mut doc).err(),
            "4" => {
                print_env_hints();
                None
            }
            "5" => match write_with_backup(&path, &doc) {
                Ok(()) => {
                    println!("Wrote {}", path.display());
                    return ExitCode::SUCCESS;
                }
                Err(e) => Some(e),
            },
            "6" => {
                println!("Aborted.");
                return ExitCode::SUCCESS;
            }
            "" => None,
            other => Some(format!("unknown option: {other}")),
        };
        if let Some(e) = err {
            eprintln!("{e}");
        }
    }

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn ensure_table_inserts_llm() {
        let mut doc = DocumentMut::new();
        set_string(&mut doc, "llm", "provider", "ollama");
        assert!(doc.to_string().contains("provider"));
        let cfg: AppConfig = toml::from_str(&doc.to_string()).expect("parse");
        assert_eq!(cfg.llm.provider.as_deref(), Some("ollama"));
    }

    #[test]
    fn write_roundtrip_project_file() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("cantrik.toml");
        let mut doc = DocumentMut::new();
        set_string(&mut doc, "llm", "provider", "anthropic");
        set_string(&mut doc, "llm", "model", "claude-3-5-sonnet-20241022");
        write_with_backup(&path, &doc).expect("write");
        let loaded = std::fs::read_to_string(&path).expect("read");
        let cfg: AppConfig = toml::from_str(&loaded).expect("parse");
        assert_eq!(cfg.llm.provider.as_deref(), Some("anthropic"));
    }
}
