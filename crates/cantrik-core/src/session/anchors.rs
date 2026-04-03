//! Load `~/.config/cantrik/anchors.md` + optional `<project>/.cantrik/anchors.md`.

use std::path::Path;

use super::paths::{global_anchors_path, project_anchors_path};

pub fn load_anchors_combined(project_root: &Path) -> String {
    let mut parts = Vec::new();
    let g = global_anchors_path();
    if g.exists()
        && let Ok(s) = std::fs::read_to_string(&g)
        && !s.trim().is_empty()
    {
        parts.push(format!("## Global anchors ({})\n{s}", g.display()));
    }
    let p = project_anchors_path(project_root);
    if p.exists()
        && let Ok(s) = std::fs::read_to_string(&p)
        && !s.trim().is_empty()
    {
        parts.push(format!("## Project anchors ({})\n{s}", p.display()));
    }
    parts.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn combines_global_and_project_anchors() {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = std::env::temp_dir().join(format!("cantrik-anchor-test-{id}"));
        let config_dir = base.join("home/.config/cantrik");
        let proj = base.join("proj");
        let dot = proj.join(".cantrik");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::create_dir_all(&dot).unwrap();
        std::fs::write(config_dir.join("anchors.md"), "global line\n").unwrap();
        std::fs::write(dot.join("anchors.md"), "project line\n").unwrap();

        let old = std::env::var_os("HOME");
        unsafe { std::env::set_var("HOME", base.join("home")) };
        let out = load_anchors_combined(&proj);
        match old {
            Some(h) => unsafe { std::env::set_var("HOME", h) },
            None => unsafe { std::env::remove_var("HOME") },
        }
        std::fs::remove_dir_all(&base).unwrap();

        assert!(out.contains("global line"));
        assert!(out.contains("project line"));
    }
}
