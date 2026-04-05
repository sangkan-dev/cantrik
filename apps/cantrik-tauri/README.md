# Cantrik Tauri tray (v2 MVP)

System tray app with **Quit** only; the main window stays hidden. Use this when you want a native tray entry without the separate [`cantrik-tray`](../cantrik-tray/) notifier.

## Build

Prerequisites: [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) (Rust, Node, system webview libraries).

```bash
cd apps/cantrik-tauri
npm install
npm run build          # frontend → dist/
cd src-tauri && cargo build   # or from repo root: same, after npm run build
```

Full bundle:

```bash
cd apps/cantrik-tauri
npm run tauri build
```

The Rust crate lives under `src-tauri/` and is listed in the root workspace **`exclude`** so it keeps its own `Cargo.lock`.

Approval notifications for background jobs still use the paths from [`crates/cantrik-core/src/background/mod.rs`](../../crates/cantrik-core/src/background/mod.rs) (`notification_channels_from_config`); this tray app does not poll flags yet—that remains the role of `cantrik-tray` until wired here.
