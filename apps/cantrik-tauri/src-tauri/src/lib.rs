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

            // Same default flag path as `cantrik-tray` / `[background].approval_flag_path` default.
            std::thread::spawn(|| {
                use std::path::PathBuf;
                use std::time::Duration;
                let flag_path = dirs::data_local_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("cantrik")
                    .join("approval-pending.flag");
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
