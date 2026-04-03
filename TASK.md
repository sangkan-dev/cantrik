# TASK.md - Cantrik Sprint Board

Dokumen ini dipakai untuk tracking implementasi berdasarkan PRD (`prd/cantrik-doc.js`) dengan model sprint dan checklist.

## Legend

- `[ ]` belum dikerjakan
- `[/]` sedang berjalan
- `[x]` selesai

## Baseline (Sudah Ada)

- [x] Inisialisasi project Rust (`Cargo.toml`, `crates/cantrik-cli/src/main.rs`)
- [x] Draft PRD tersedia di folder `prd/`

## Sprint Plan

2 minggu per sprint.

## Sprint 1 - Fondasi Proyek & Tooling

Goal: Menyiapkan pondasi engineering yang rapi dan repeatable.

- [x] Setup struktur workspace Rust (multi-crate ready)
- [x] Tambahkan dependencies inti (`tokio`, `clap`, `serde`, `toml`, `reqwest`)
- [x] Setup `rustfmt` + `clippy` + pre-commit checks
- [x] Setup CI GitHub Actions (build, test, clippy, fmt)
- [x] Buat skeleton config loader (`~/.config/cantrik` + project `.cantrik`)

**Definition of Done:**

Goal: CLI bisa dipakai untuk mode dasar.

- [ ] Implement `clap` parser + subcommand utama (`ask`, `plan`, `index`, `doctor`)
- [ ] Implement help text dan UX command konsisten
- [ ] Tambahkan shell completion generation
- [ ] Buat mode one-shot (`cantrik "..."`) dan REPL placeholder

**Definition of Done:**] Implement help text dan UX command konsisten
- [ ] Tambahkan shell completion generation
- [ ] Buat mode one-shot (`cantrik "..."`) dan REPL placeholder
DoD:

Goal: Integrasi provider utama dengan interface seragam.

- [ ] Desain trait/provider abstraction LLM
- [ ] Implement provider Anthropic
- [ ] Implement provider Gemini
- [ ] Implement provider Ollama
- [ ] Implement streaming response ke terminal
- [ ] Implement fallback chain sederhana

**Definition of Done:**] Implement provider Ollama
- [ ] Implement streaming response ke terminal
- [ ] Implement fallback chain sederhana
DoD:

Goal: Pengalaman interactive CLI sudah usable.

- [ ] Integrasi `ratatui` + `crossterm` untuk REPL
- [ ] Render thinking log dan output streaming
- [ ] Implement input history + session state in-memory
- [ ] Implement `/cost`, `/memory`, `/doctor` (minimal)

**Definition of Done:**] Render thinking log dan output streaming

Goal: Cantrik paham struktur codebase secara semantik.

- [ ] Integrasi `tree-sitter` (Rust, JS/TS, Python, Go)
- [ ] Implement AST-aware chunking
- [ ] Implement file scanner (`.gitignore`-aware)
- [ ] Implement incremental re-index (berbasis perubahan file)

**Definition of Done:**print 5 - Codebase Intelligence (AST & Indexing)
Goal: Cantrik paham struktur codebase secara semantik.
- [ ] Integrasi `tree-sitter` (Rust, JS/TS, Python, Go)
- [ ] Implement AST-aware chunking

Goal: Cari konteks relevan berbasis embedding lokal.

- [ ] Integrasi LanceDB (embedded)
- [ ] Pipeline embedding default via Ollama (`nomic-embed-text`)
- [ ] Simpan metadata chunk (path, symbol, language)
- [ ] Implement semantic search API + CLI command

**Definition of Done:**print 6 - Vector Memory & Semantic Search
Goal: Cari konteks relevan berbasis embedding lokal.
- [ ] Integrasi LanceDB (embedded)
- [ ] Pipeline embedding default via Ollama (`nomic-embed-text`)

Goal: Memori sesi persist dan context tidak meledak.

- [ ] Setup SQLite untuk history sesi + ringkasan
- [ ] Simpan keputusan penting per sesi
- [ ] Implement context pruning + summarization saat window penuh
- [ ] Support memory anchors dari file config

**Definition of Done:**print 7 - Session Memory & Context Compression

Goal: Eksekusi aksi dengan kontrol izin yang aman.

- [ ] Implement tool registry (`read_file`, `write_file`, `run_command`, `search`)
- [ ] Implement permission tiers (forbidden/approval/auto)
- [ ] Implement approval prompt sebelum write/exec/network
- [ ] Implement sandbox level `restricted` (minimum viable)

**Definition of Done:**
- [ ] Session restart tetap bisa recover konteks inti

## Sprint 8 - Tool System & Guardrails v1

Goal: Perubahan bisa dilacak dan dibatalkan dengan aman.

- [ ] Auto checkpoint sebelum operasi write
- [ ] Implement command `rollback` + list checkpoint
- [ ] Implement audit log append-only
- [ ] Tambah basic cost tracking per aksi/model

**Definition of Done:**
- [ ] Operasi berisiko selalu minta approval dan tercatat

## Sprint 9 - Checkpointing, Rollback, Audit Log
Goal: Perubahan bisa dilacak dan dibatalkan dengan aman.
- [ ] Auto checkpoint sebelum operasi write
- [ ] Implement command `rollback` + list checkpoint
- [ ] Implement audit log append-only
- [ ] Tambah basic cost tracking per aksi/model
DoD:
- [ ] Satu perubahan file bisa rollback sempurna

## Sprint 10 - Agentic Execution & Re-planning
Goal: Mampu menjalankan task multi-step dengan fallback.
- [ ] Implement plan engine (`plan -> act -> evaluate`)
- [ ] Implement stuck detection (threshold percobaan)
- [ ] Human escalation message saat gagal berulang
- [ ] Integrasi plan mode ke command `--plan`
DoD:
- [ ] Task kompleks bisa jalan multi-step dengan jalur recovery

## Sprint 11 - Multi-agent v1
Goal: Sub-agent paralel untuk percepat analisis.
- [ ] Implement orchestrator + sub-agent context terpisah
- [ ] Implement parallel execution (`tokio::spawn`)
- [ ] Implement summary propagation antar agent
- [ ] Batasi depth spawn agent (default 3)
DoD:
- [ ] Task dekomposisi 3 sub-agent selesai lebih cepat dari serial

## Sprint 12 - Plugin System (Skill + Lua + WASM)
Goal: Extensibility untuk workflow custom.
- [ ] Implement auto-inject skill files (`.cantrik/skills/*.md`)
- [ ] Integrasi runtime Lua (`mlua`) untuk plugin ringan
- [ ] Integrasi runtime WASM (`wasmtime`) untuk plugin advanced
- [ ] Implement command install/list/update plugin (local registry dulu)
DoD:
- [ ] Minimal 1 plugin Lua dan 1 plugin WASM berjalan

## Sprint 13 - Git-native Workflow & PR Automation
Goal: Workflow coding ke PR lebih otomatis.
- [ ] Auto-branch per task
- [ ] Generate commit message berbasis semantic summary
- [ ] Integrasi create PR (GitHub/GitLab)
- [ ] Basic conflict detection + rekomendasi resolusi
DoD:
- [ ] Dari task ke PR bisa dieksekusi end-to-end pada repo demo

## Sprint 14 - MCP + Background Mode
Goal: Integrasi eksternal dan long-running task.
- [ ] Implement mode `cantrik serve --mcp` (server)
- [ ] Implement consume MCP server lain (client)
- [ ] Implement background daemon + status command
- [ ] Implement notifikasi saat butuh approval
DoD:
- [ ] Background task tetap lanjut setelah terminal ditutup

## Sprint 15 - UX Lanjut, Voice, Visual Intelligence
Goal: Diferensiasi pengalaman pakai Cantrik.
- [ ] Semantic diff + risk assessment output
- [ ] Voice-to-code (Whisper lokal) + TTS notifikasi
- [ ] `/visualize` untuk Mermaid/PlantUML
- [ ] Web research tool dengan approval eksplisit
DoD:
- [ ] User bisa meninjau semantic diff sebelum apply

## Sprint 16 - Ecosystem & Distribution
Goal: Siap adopsi komunitas open source.
- [ ] Siapkan packaging (Homebrew, deb/apt, pacman, Nix, winget)
- [ ] Siapkan docs kontribusi + quality bar (>=80% core coverage)
- [ ] Siapkan website/hub plugin (`cantrik.dev` placeholder)
- [ ] Rancang VS Code extension dan desktop companion scope
DoD:
- [ ] Rilis alpha publik + checklist kontribusi jelas

## Backlog (Belum Dijadwalkan)
- [ ] Full autonomous SWE-agent mode
- [ ] Self-improvement loop untuk codebase Cantrik
- [ ] Benchmark formal vs SWE-bench
- [ ] Air-gapped enterprise mode

## Catatan Operasional
- Update status tiap selesai PR: ubah `[ ]` -> `[/]` -> `[x]`
- Satu sprint boleh split jadi beberapa PR kecil
- Jika scope sprint meleset >30%, pindahkan item ke sprint berikutnya dan beri catatan alasan
