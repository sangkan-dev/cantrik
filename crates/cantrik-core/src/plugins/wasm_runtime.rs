//! WASM plugins under `.cantrik/plugins/*.wasm` (Sprint 13, PRD §7 layer 3).

use std::fs;
use std::path::Path;

use wasmtime::{Engine, Linker, Module, Store};

use super::lua_runtime::ENV_NO_PLUGINS;

/// Export name: host calls this after a successful file write (no arguments). Return value ignored.
pub const WASM_HOOK_EXPORT: &str = "after_write_ping";

/// Load each `*.wasm` and call [`WASM_HOOK_EXPORT`] if present (no WASI; module must need no imports).
pub fn run_wasm_after_write_hooks(project_root: &Path) {
    if std::env::var(ENV_NO_PLUGINS).is_ok() {
        return;
    }
    let dir = project_root.join(".cantrik").join("plugins");
    let Ok(rd) = fs::read_dir(&dir) else {
        return;
    };
    let mut paths: Vec<_> = rd
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "wasm"))
        .collect();
    paths.sort();
    for p in paths {
        if let Err(e) = invoke_wasm_hook(&p) {
            eprintln!("cantrik wasm plugin {}: {e}", p.display());
        }
    }
}

fn invoke_wasm_hook(wasm_path: &Path) -> Result<(), String> {
    let engine = Engine::default();
    let wasm = fs::read(wasm_path).map_err(|e| e.to_string())?;
    let module = Module::new(&engine, &wasm).map_err(|e| e.to_string())?;
    let mut store = Store::new(&engine, ());
    let linker = Linker::new(&engine);
    let instance = linker
        .instantiate(&mut store, &module)
        .map_err(|e| e.to_string())?;
    let Ok(f) = instance.get_typed_func::<(), i32>(&mut store, WASM_HOOK_EXPORT) else {
        return Ok(());
    };
    f.call(&mut store, ()).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wasm_load_and_ping() {
        let wasm = wat::parse_str(
            r#"(module
              (func (export "after_write_ping") (result i32)
                (i32.const 42))
            )"#,
        )
        .expect("wat");
        let engine = Engine::default();
        let module = Module::new(&engine, &wasm).expect("module");
        let mut store = Store::new(&engine, ());
        let linker = Linker::new(&engine);
        let instance = linker.instantiate(&mut store, &module).expect("instance");
        let f = instance
            .get_typed_func::<(), i32>(&mut store, WASM_HOOK_EXPORT)
            .expect("func");
        let v = f.call(&mut store, ()).expect("call");
        assert_eq!(v, 42);
    }
}
