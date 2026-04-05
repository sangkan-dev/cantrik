//! Tauri tray MVP: poll the same approval flag file as `cantrik-tray` / daemon notifications.

use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Full path override (highest priority).
pub const ENV_APPROVAL_FLAG_PATH: &str = "CANTRIK_APPROVAL_FLAG_PATH";

/// Optional project root: when set, merge `.cantrik/cantrik.toml` over global (same idea as CLI).
pub const ENV_PROJECT_ROOT: &str = "CANTRIK_PROJECT_ROOT";

#[derive(Debug, Deserialize, Default)]
struct BackgroundToml {
    approval_flag_path: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct PartialConfigToml {
    #[serde(default)]
    background: BackgroundToml,
}

fn default_share_flag_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cantrik")
        .join("approval-pending.flag")
}

fn global_config_file() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join(".config")
        .join("cantrik")
        .join("config.toml")
}

fn read_partial_config(path: &Path) -> Option<PartialConfigToml> {
    let s = std::fs::read_to_string(path).ok()?;
    toml::from_str(&s).ok()
}

fn flag_from_config(cfg: &PartialConfigToml) -> Option<PathBuf> {
    cfg.background
        .approval_flag_path
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
}

/// Resolve flag path: env → project `cantrik.toml` (if `CANTRIK_PROJECT_ROOT`) → global `config.toml` → default share dir.
pub fn resolve_approval_flag_path() -> PathBuf {
    if let Ok(p) = std::env::var(ENV_APPROVAL_FLAG_PATH) {
        let pb = PathBuf::from(p.trim());
        if !pb.as_os_str().is_empty() {
            return pb;
        }
    }

    let global_flag = read_partial_config(&global_config_file()).and_then(|c| flag_from_config(&c));

    let project_flag = std::env::var(ENV_PROJECT_ROOT).ok().and_then(|root| {
        let cfg = PathBuf::from(root.trim())
            .join(".cantrik")
            .join("cantrik.toml");
        read_partial_config(&cfg).and_then(|c| flag_from_config(&c))
    });

    project_flag
        .or(global_flag)
        .unwrap_or_else(default_share_flag_path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            use tauri::menu::{Menu, MenuItem};
            use tauri::tray::TrayIconBuilder;

            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;
            let icon = app
                .default_window_icon()
                .expect("bundle should include default window icons")
                .clone();
            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| {
                    if event.id.as_ref() == "quit" {
                        app.exit(0);
                    }
                })
                .build(app)?;

            let flag_path = resolve_approval_flag_path();
            std::thread::spawn(move || {
                use std::time::Duration;
                let mut was_present = flag_path.exists();
                loop {
                    let present = flag_path.exists();
                    if present && !was_present {
                        let body = std::fs::read_to_string(&flag_path)
                            .map(|s| format!("Job id / hint: {}", s.trim()))
                            .unwrap_or_else(|_| {
                                "Approval pending — see `cantrik background`.".to_string()
                            });
                        let _ = notify_rust::Notification::new()
                            .summary("Cantrik — approval needed")
                            .body(&body)
                            .show();
                    }
                    was_present = present;
                    std::thread::sleep(Duration::from_secs(8));
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
