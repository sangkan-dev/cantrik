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

A background thread polls the default approval flag under the OS data directory (`…/cantrik/approval-pending.flag`), same default as [`cantrik-tray`](../cantrik-tray/). Override path in project config via `[background].approval_flag_path` is **not** read by this app yet (use `cantrik-tray` for custom paths until wired).
