//! Project Lua plugins under `.cantrik/plugins/*.lua` (Sprint 13, PRD §7 layer 2).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use mlua::{Lua, Result as LuaResult};

/// Disable Lua/WASM plugin hooks (tests).
pub const ENV_NO_PLUGINS: &str = "CANTRIK_NO_PLUGINS";

pub fn plugins_dir(cwd: &Path) -> PathBuf {
    cwd.join(".cantrik").join("plugins")
}

fn register_cantrik(lua: &Lua, suggests: Arc<Mutex<Vec<String>>>) -> LuaResult<()> {
    let cantrik = lua.create_table()?;
    let s = suggests.clone();
    cantrik.set(
        "suggest",
        lua.create_function(move |_, msg: String| {
            if let Ok(mut g) = s.lock() {
                g.push(msg);
            }
            Ok(())
        })?,
    )?;
    cantrik.set(
        "log",
        lua.create_function(|_, msg: String| {
            eprintln!("cantrik plugin [log]: {msg}");
            Ok(())
        })?,
    )?;
    cantrik.set(
        "warn",
        lua.create_function(|_, msg: String| {
            eprintln!("cantrik plugin [warn]: {msg}");
            Ok(())
        })?,
    )?;
    cantrik.set(
        "require_approval",
        lua.create_function(|_, hint: String| {
            eprintln!("cantrik plugin [require_approval] (stub): {hint}");
            Ok(())
        })?,
    )?;
    lua.globals().set("cantrik", cantrik)?;
    Ok(())
}

fn run_hook_in_file(path: &Path, hook: &str, arg: &str) -> LuaResult<Vec<String>> {
    let lua = Lua::new();
    let suggests = Arc::new(Mutex::new(Vec::<String>::new()));
    register_cantrik(&lua, suggests.clone())?;
    let src = fs::read_to_string(path)?;
    lua.load(src).set_name(path.to_string_lossy()).exec()?;
    let globals = lua.globals();
    let f: Option<mlua::Function> = globals.get(hook)?;
    if let Some(f) = f {
        f.call::<()>(arg)?;
    }
    Ok(suggests.lock().map(|g| g.clone()).unwrap_or_default())
}

fn foreach_lua_file(cwd: &Path, mut f: impl FnMut(&Path)) {
    if std::env::var(ENV_NO_PLUGINS).is_ok() {
        return;
    }
    let dir = plugins_dir(cwd);
    let Ok(rd) = fs::read_dir(&dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = rd
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "lua"))
        .collect();
    paths.sort();
    for p in paths {
        f(&p);
    }
}

/// Run `on_task_start(task)` from each `.lua` plugin; collect `cantrik.suggest` messages.
pub fn on_task_start_messages(cwd: &Path, task: &str) -> Vec<String> {
    let mut out = Vec::new();
    foreach_lua_file(cwd, |path| {
        match run_hook_in_file(path, "on_task_start", task) {
            Ok(v) => out.extend(v),
            Err(e) => eprintln!("cantrik lua plugin {}: {e}", path.display()),
        }
    });
    out
}

/// Run `after_write(rel_path)` from each `.lua` plugin; collect suggestions.
pub fn after_write_messages(cwd: &Path, rel_path: &str) -> Vec<String> {
    let mut out = Vec::new();
    foreach_lua_file(cwd, |path| {
        match run_hook_in_file(path, "after_write", rel_path) {
            Ok(v) => out.extend(v),
            Err(e) => eprintln!("cantrik lua plugin {}: {e}", path.display()),
        }
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lua_on_task_start_suggest() {
        let dir = std::env::temp_dir().join(format!("cantrik-lua-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(plugins_dir(&dir)).unwrap();
        fs::write(
            plugins_dir(&dir).join("t.lua"),
            r#"
function on_task_start(task)
  cantrik.suggest("check: " .. task)
end
"#,
        )
        .unwrap();
        let msgs = on_task_start_messages(&dir, "hello");
        assert_eq!(msgs, vec!["check: hello"]);
        let _ = fs::remove_dir_all(&dir);
    }
}
