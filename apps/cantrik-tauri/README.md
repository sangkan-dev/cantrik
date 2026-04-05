# Cantrik Tauri tray (scaffold)

Full **Tauri v2** tray UI is tracked under backlog Phase 4 (replace or wrap [`cantrik-tray`](../cantrik-tray/)).

Until then:

1. Run the lightweight notifier: `cd ../cantrik-tray && cargo run` (polls `~/.local/share/cantrik/approval-pending.flag`).
2. To scaffold Tauri here: install [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/), then `npm create tauri-app@latest` in an empty subfolder and port the same polling + `notify-rust` logic into `setup()` with a tray icon.

Reuse the default approval flag path from [`crates/cantrik-core/src/background/mod.rs`](../../crates/cantrik-core/src/background/mod.rs) (`notification_channels_from_config`).
