# TASK.md - Cantrik Sprint Board

Dokumen ini dipakai untuk tracking implementasi berdasarkan **PRD** di `prd/cantrik-prd.md` (bagian *Roadmap Pengembangan*, fitur inti, arsitektur). Satu sprint diasumsikan ~2 minggu.

## Legend

- `[ ]` belum dikerjakan
- `[/]` sedang berjalan
- `[x]` selesai

## Pemetaan PRD → Sprint (ringkas)

| Fase PRD (`cantrik-prd.md`) | Sasaran | Sprint di board |
|----------------------------|---------|-----------------|
| **Phase 0** — Fondasi (Bulan 1–2) | CLI dasar, bridge LLM, REPL awal, config | 1–4 |
| **Phase 1** — Core Intelligence (Bulan 3–4) | AST + vektor + memori sesi + alat file | 5–7 |
| **Phase 2** — Agentic (Bulan 5–6) | Tool eksekusi, checkpoint/audit, re-plan, multi-agent | 8–11 |
| **Phase 3** — Advanced (Bulan 7–9) | Daemon, plugin, routing biaya, MCP, diff semantik, kolaborasi, suara, Git/PR, web, LSP/visual, macro/rules | 12–17 |
| **Phase 4** — Ekosistem (Bulan 10–12) | Hub, template, air-gap, distribusi, VS Code, Tauri, tech debt & adaptive learning | 18 |
| **Phase 5** — Maturity | Mode SWE otonom penuh, self-improve, benchmark | Backlog |

## Baseline (Sudah Ada)

- [x] Inisialisasi project Rust (`Cargo.toml`, `crates/cantrik-cli/src/main.rs`)
- [x] Draft PRD tersedia di folder `prd/` (`cantrik-prd.md`)

---

## Sprint 1 — Fondasi proyek & tooling (Phase 0)

**Goal:** Pondasi engineering yang rapi dan repeatable.

- [x] Setup struktur workspace Rust (multi-crate)
- [x] Dependencies inti (`tokio`, `clap`, `serde`, `toml`, `reqwest`, `thiserror`)
- [x] `rustfmt` + `clippy` (-D warnings) + pre-commit checks
- [x] CI GitHub Actions (build, test, clippy, fmt)
- [x] Skeleton config: merge `~/.config/cantrik/config.toml` + `.cantrik/cantrik.toml` (precedence project > global)

**Definition of Done:** `cargo fmt`, `clippy`, `test`, dan CI hijau; config loader teruji.

---

## Sprint 2 — CLI scaffold & permukaan perintah (Phase 0)

**Goal:** Struktur `clap` dan UX perintah konsisten sesuai PRD (subcommand + mode input dasar).

- [x] Parser `clap` + subcommand utama: `ask`, `plan`, `index`, `doctor` (+ `--help` konsisten)
- [x] Mode one-shot: `cantrik "..."` (alias ke `ask` via `external_subcommand` — argumen tidak bentrok dengan nama subcommand)
- [x] REPL / interactive: `cantrik` tanpa subcommand di TTY — placeholder loop (`exit`/`quit`/EOF); TUI penuh di Sprint 4
- [x] Shell completion generation (`clap_complete`), subcommand `completions <bash|zsh|fish|elvish|powershell>`
- [x] Pipe dari stdin ke `ask` ketika stdin bukan TTY (batas ~4 MiB)
- [x] Flag tersembunyi stub: `--watch`, `--from-clipboard`, `--image` → pesan + exit code 2 (belum diimplementasi)
- **Deferred:** flag root `--plan "…"` ala PRD; gunakan **`cantrik plan "…"`** sampai flag global ditambahkan. Integrasi clipboard/watch/vision sesungguhnya di sprint berikutnya.

**Definition of Done:** Pengguna bisa memanggil tiap subcommand utama, help jelas, completion bisa di-generate; satu path one-shot dan satu path REPL jalan.

---

## Sprint 3 — LLM bridge v1 (Phase 0)

**Goal:** Multi-provider dengan antarmuka seragam, streaming, fallback — selaras *LLM Bridge* + *Provider Matrix* PRD (inti: Anthropic, Gemini, Ollama).

- [x] Abstraksi LLM di `cantrik-core::llm`: `ask_stream_chunks` + orkestrasi async (stream per provider, bukan trait object — cukup untuk v1)
- [x] Provider: Anthropic (Messages + SSE), Gemini (`streamGenerateContent` REST), Ollama (`/api/chat` NDJSON stream)
- [x] Streaming ke stdout dari `cantrik ask`, `plan`, stdin/eksternal; stderr untuk error
- [x] `[routing].fallback_chain` di `providers.toml` + target primer dari `cantrik.toml` `[llm]`; percobaan berurutan (abort fallback jika sudah ada output)
- [x] `~/.config/cantrik/providers.toml` + `api_key` / `${VAR}` + fallback ke `ANTHROPIC_API_KEY` / `GEMINI_API_KEY`; `doctor` menampilkan status tanpa secret
- [x] Provider tambahan PRD: **OpenAI**, **Azure OpenAI** (deployment + `api-version`), **OpenRouter**, **Groq** — streaming via Chat Completions SSE (kompatibel OpenAI)

**Definition of Done:** Minimal satu model per tiga provider utama bisa chat non-interaktif dengan streaming; fallback bisa dikonfigurasi.

**Verifikasi:** uji manual per provider (API key / Ollama lokal); CI hanya tes parsing + rantai fallback (tanpa jaringan).

**Contoh `providers.toml` (cuplikan):**

```toml
[providers.openai]
api_key = "${OPENAI_API_KEY}"
default_model = "gpt-4o-mini"

[providers.azure]
api_key = "${AZURE_OPENAI_API_KEY}"
endpoint = "https://YOUR_RESOURCE.openai.azure.com"
default_deployment = "gpt-4o"
api_version = "2024-02-01-preview"

[providers.openrouter]
api_key = "${OPENROUTER_API_KEY}"
default_model = "anthropic/claude-3.5-sonnet"

[providers.groq]
api_key = "${GROQ_API_KEY}"
default_model = "llama-3.3-70b-versatile"
```

---

## Sprint 4 — REPL dasar & TUI (Phase 0)

**Goal:** *Basic REPL* PRD: `ratatui` + `crossterm`, log berpikir, perintah built-in awal.

- [x] Integrasi `ratatui` + `crossterm`
- [x] Render *thinking log* + output streaming (sesuai gaya *Terminal UX* PRD)
- [x] Riwayat input + state sesi in-memory
- [x] Perintah built-in minimal: `/cost`, `/memory`, `/doctor` (sesuai tabel *Built-in Commands* PRD)

**Definition of Done:** REPL bisa sesi percakapan singkat dengan log dan tiga perintah di atas.

**Catatan:** `/cost` dan `/memory` berupa stub/jelasan tier sesuai PRD (pelacakan biaya nyata di Sprint 14+; persistensi memori di Sprint 6–7). `/doctor` memakai `doctor::report_lines` yang sama dengan subcommand `cantrik doctor`.

---

## Sprint 5 — Codebase intelligence: AST & indexing (Phase 1)

**Goal:** Pemahaman struktur kode selaras *Codebase Intelligence* PRD.

- [x] Integrasi `tree-sitter` — Phase 1 PRD: Rust, Python, JS/TS/TSX, Go, Java, C/C++ (`.c`/`.h` vs `.cpp`/`.cc`/…), PHP, Ruby, SQL (`tree-sitter-sequel`), TOML (`tree-sitter-toml-ng`), JSON, YAML, Markdown (`tree-sitter-md` blok)
- [x] AST-aware chunking (batas fungsi/class, bukan potong karakter naif)
- [x] *Dependency graph* (siapa memanggil siapa) — sesuai fitur inti PRD
- [x] File scanner `.gitignore`-aware + batas ukuran/file biner
- [x] Re-index inkremental (hanya file berubah)

**Definition of Done:** Index folder proyek menghasilkan chunk AST + metadata path/symbol; scan menghormati `.gitignore`.

**Catatan:** Artefak di `.cantrik/index/ast/` (`manifest.json`, `chunks.jsonl`, `graph.json`). Graf v1 hanya **intra-file** (nama callee dari AST panggilan; tanpa resolusi import/symbol lintas file). Sprint 6 (LanceDB) dapat memakai direktori `.cantrik/index/` untuk vektor. SQL/Markdown/TOML memakai grammar crate yang selaras **tree-sitter 0.26**; chunk SQL/Markdown bisa kasar (per-statement / heading+code fence); data non-kode (JSON/YAML/TOML) berbasis struktur parse, bukan “fungsi”.

---

## Sprint 6 — Vector store & pencarian semantik (Phase 1)

**Goal:** *Tier 3 Project Memory* PRD — LanceDB embedded, embedding lokal.

- [x] Integrasi LanceDB (embedded) di `.cantrik/index/lance/` (selaras *Directory Structure* PRD; AST tetap di `ast/`)
- [x] Pipeline embedding default Ollama HTTP `/api/embed` (`nomic-embed-text`); konfigurasi `[index]` (`vector_model`, `ollama_base`)
- [x] Metadata chunk (path, symbol, bahasa, kind, byte/row anchors, preview, `content_hash`, `chunk_id`)
- [x] Semantic search: `cantrik_core::search::{build_vector_index, semantic_search}` + CLI `cantrik search` + `cantrik index` (default jalankan vektor; `--no-vectors` opt-out)
- [x] `doctor`: status baris LanceDB + opsi config index; CI: `protobuf-compiler` untuk build `lance-encoding`

**Definition of Done:** Query teks mengembalikan chunk relevan dari index lokal tanpa kirim kode ke cloud hanya untuk embedding default.

**Catatan / backlog:** embedding cloud (OpenAI/Azure, dll.) tidak wajib Sprint 6 — lanjut sprint berikutnya bila PRD menghendaki.

**Build:** dependensi Lance membutuhkan `protoc` + include well-known types (`PROTOC_INCLUDE` jika perlu); lihat langkah CI `apt-get install protobuf-compiler`.

---

## Sprint 7 — Session memory & alat file (Phase 1)
 
**Goal:** Session Memory + File tools PRD (SQLite/sqlx, ringkasan, pruning, anchors).

**Batas MVP:** Pruning memakai heuristik token/char (bukan tiktoken penuh). Tier 4 hanya skeleton di DB (`adaptive_stub`); pembelajaran adaptif di Sprint 19.
 
- [x] SQLite untuk histori sesi + ringkasan (`~/.local/share/cantrik/memory.db`)
- [x] Simpan keputusan penting per sesi; query sesi sebelumnya
- [x] Context pruning + hierarchical summarization saat window penuh (§4.6 PRD)
- [x] Memory anchors (`anchors.md` global + opsi proyek) — always injected
- [x] Tool: `read_file`, `write_file` dengan diff preview + approval sebelum tulis
- [x] Tier 4 Global Memory skeleton — stub untuk Adaptive Learning (implementasi penuh di Sprint 19)
 
**Definition of Done:** Sesi bisa dilanjutkan dengan ringkasan; tulis file tidak tanpa preview/approve; anchor ikut dimuat ke konteks.
 
---
 
## Sprint 8 — Tool system & sandbox (Phase 2)
 
**Goal:** Eksekusi aman — Sandboxed Execution + Permission Tiers PRD.

**Batas MVP:** `container` sandbox belum; macOS `restricted` membutuhkan `CANTRIK_SANDBOX=0` atau bubblewrap tidak dipakai (pesan jelas); LLM tool-calling loop menyusul sprint berikutnya.
 
- [x] Registry tool: `run_command`, `search`/grep, `read_file`/`write_file` (integrasi penuh dengan tier)
- [x] Tier: forbidden / require_approval / auto_approve (§5 PRD)
- [x] Prompt approval untuk write, exec, network
- [x] Sandbox level `restricted` minimum viable (bubblewrap Linux / sandbox-exec macOS)
- [x] `git_ops` read-only + `web_fetch` opsional dengan approval
 
**Definition of Done:** Tidak ada write/exec/network tanpa jalur approval; sandbox default aktif untuk exec.
 
---
 
## Sprint 9 — Checkpoint, rollback, audit (Phase 2)
 
**Goal:** Checkpointing & Rollback + Audit Log PRD (§4.5, §5).
 
- [x] Auto checkpoint sebelum operasi write (`.cantrik/checkpoints/`)
- [x] Perintah `rollback` + `rollback --list` + `rollback <id>`
- [x] Audit log append-only (`~/.local/share/cantrik/audit.log`) sesuai contoh PRD
- [x] Cost tracking per aksi / model — disiapkan untuk `/cost` command
- [x] Provenance metadata per baris kode yang ditulis Cantrik (§4.10 PRD) — `.cantrik/provenance.jsonl` (file-first; inline comment ditunda)
 
**Definition of Done:** Satu alur tulis file bisa di-rollback; aksi tercatat di audit dengan cost.
 
**Batas MVP Sprint 9:** Harga API riil dan agregasi `/cost` belum — field `cost` di audit stub (`0.0`); provenance via `provenance.jsonl` + `[audit] provenance = "off"`; `CANTRIK_AUDIT_LOG` override path audit; multi-file checkpoint tunggal per write (bukan transaksi batch).
 
---
 
## Sprint 10 — Planning, re-planning & escalation (Phase 2)
 
**Goal:** Long-horizon Planning + Stuck Detection PRD (§4.4).
 
- [x] Mesin plan → act → evaluate; re-plan jika langkah gagal
- [x] Deteksi stuck (threshold default: 3 percobaan berbeda)
- [x] Eskalasi ke user dengan ringkasan percobaan yang sudah dilakukan
- [x] Integrasi ke subcommand `cantrik plan` dan perintah `/plan`
- [x] Experiment Mode (§4.21): eksekusi perubahan, run test/benchmark, auto-revert jika tidak ada improvement
 
**Definition of Done:** Task multi-step bisa re-plan atau berhenti dengan pesan eskalasi jelas; experiment mode bisa revert otomatis.
 
**Batas MVP Sprint 10:** “Act” pada loop plan memakai ringkasan simulasi (bukan eksekusi tool otomatis); evaluasi langkah memakai LLM + JSON; `cantrik plan --run` dan REPL `/plan` tanpa `--run` (generate + simpan state); experiment = JSON `writes` + exit code perintah `[planning].experiment_test_command` (default `cargo test`), rollback memanggil `revert_checkpoints_after_seq`; benchmark numerik ditunda.
 
---
 
## Sprint 11 — Multi-agent v1 (Phase 2)
 
**Goal:** Multi-Agent Orchestration PRD (§4.2).
 
- [ ] Orchestrator + konteks sub-agent terpisah (isolated context window)
- [ ] Eksekusi paralel via `tokio`
- [ ] Summary propagation ke orchestrator (hemat token)
- [ ] Batas kedalaman spawn (default: 3)
- [ ] Failure isolation — satu sub-agent gagal tidak stop yang lain
- [ ] Structured Plan & Act Mode — stub awal: Planner (read-only) + Builder (approval) (§4.12 PRD)
 
**Definition of Done:** Task terdekomposisi ke beberapa sub-agent paralel; Planner dapat berjalan tanpa akses write.
 
---
 
## Sprint 12 — Background agent & daemon (Phase 3)
 
**Goal:** Background Agent Mode PRD (§4.3).
 
- [ ] Mode background / long-running + persist progress ke SQLite
- [ ] Integrasi daemon: systemd user service (Linux) / launchd (macOS)
- [ ] Notifikasi saat perlu approval: desktop (notify-send / osascript), webhook URL, file flag
- [ ] `cantrik status` — cek progress task background
 
**Definition of Done:** Task panjang tetap berjalan setelah terminal tertutup; notifikasi terkirim saat approval dibutuhkan.
 
---
 
## Sprint 13 — Plugin & skill system (Phase 3)
 
**Goal:** Tiga lapis PRD — skill `.md`, Lua `mlua`, WASM `wasmtime` (§7 PRD).
 
- [ ] Auto-inject skill (`.cantrik/skills/*.md`) berdasarkan relevansi task
- [ ] Auto-inject `.cantrik/rules.md` — always injected, berbeda dari skills (§4.19 PRD)
- [ ] Runtime Lua (`mlua`) untuk plugin proyek — hook `on_task_start`, `after_write`, dll.
- [ ] Runtime WASM (`wasmtime`) untuk plugin advanced — sandbox penuh
- [ ] Perintah `cantrik skill install/list/update/remove` (registry lokal dulu)
- [ ] Macro & Recipe System (§4.18 PRD): `cantrik macro record/stop/run`
 
**Definition of Done:** Minimal satu contoh plugin Lua dan satu WASM berjalan; rules.md selalu di-inject; satu macro bisa di-record dan di-replay.
 
---
 
## Sprint 14 — Smart routing, biaya & MCP (Phase 3)
 
**Goal:** Smart Routing + Cost Control + MCP Integration PRD (§3, §4.9).
 
- [ ] Routing model otomatis berdasarkan task complexity (simple/medium/complex threshold)
- [ ] Budget: `max_cost_per_session` dan `max_cost_per_month` dari config
- [ ] `/cost` command — tampilkan usage & biaya real per session + bulan ini
- [ ] `cantrik serve --mcp` — Cantrik sebagai MCP server
- [ ] Konsumsi MCP server eksternal (GitHub MCP, Postgres MCP, dll.) sebagai client
 
**Definition of Done:** Cantrik bisa dipanggil dari host MCP dan memanggil tools MCP lain; routing model berfungsi sesuai threshold.
 
---
 
## Sprint 15 — Semantic diff & kolaborasi (Phase 3)
 
**Goal:** Semantic Diff & Merge + Collaborative Mode PRD (§4.8, §4.23).
 
- [ ] Output semantic diff + risk assessment + fungsi/file terdampak
- [ ] Cek cakupan tes per perubahan — saran minimal heuristik
- [ ] Conflict detection Git + saran resolusi
- [ ] Export/import context (`cantrik export`, `cantrik import`)
- [ ] Context Handoff Protocol: `cantrik handoff` → `.cantrik/handoff-YYYY-MM-DD.md` (§4.23 PRD)
- [ ] Session Replay: simpan dan replay sesi (§4.27 PRD)
 
**Definition of Done:** User bisa review ringkasan perubahan semantik sebelum apply; handoff file bisa di-generate dan di-load.
 
---
 
## Sprint 16 — Git-native workflow, review & web research (Phase 3)
 
**Goal:** Deep Git-Native Workflow + `cantrik review` + Web Research PRD (§4.11, §4.13, §4.22).
 
- [ ] Auto-branch per task: `feature/cantrik-<task-slug>`
- [ ] AI-generated commit message (semantic style) + approval sebelum commit
- [ ] `cantrik pr create` — integrasi GitHub atau GitLab (minimal satu penyedia via `gh` CLI atau MCP)
- [ ] `cantrik fix <issue-url>` — SWE-agent mode: analisis issue, fix, test, buat PR (stretch — boleh defer sebagian)
- [ ] `cantrik review` — pre-commit AI review; bisa jadi git pre-commit hook
- [ ] Web research: `web_search`, `browse_page`, `fetch_docs` dengan approval eksplisit (§4.13 PRD)
 
**Definition of Done:** Alur lokal dari auto-branch hingga PR dapat diotomatisasi pada repo demo; review command bisa run standalone; web fetch hanya setelah approve.
 
---
 
## Sprint 17 — Intelligence tools: explain, teach, dependency, experiment (Phase 3)
 
**Goal:** Code archaeology, knowledge extraction, dependency intel, experiment mode PRD (§4.20–4.25).
 
- [ ] `cantrik explain [file] --why` — Code Archaeology via git blame + commit history (§4.20 PRD)
- [ ] `cantrik teach` — generate ARCHITECTURE.md, ADR, API docs dari codebase (§4.25 PRD)
- [ ] `cantrik teach --format wiki` — export ke format Obsidian/Notion/Confluence-compatible
- [ ] `cantrik why <dep>`, `cantrik upgrade`, `cantrik audit` — Dependency Intelligence (§4.24 PRD)
- [ ] Experiment Mode full implementation: run benchmark, compare, auto-revert (§4.21 PRD)
 
**Definition of Done:** Minimal `cantrik explain` dan `cantrik audit` berjalan end-to-end; experiment mode bisa revert otomatis berdasarkan hasil test.
 
---
 
## Sprint 18 — LSP, visual, voice & advanced UX (Phase 3)
 
**Goal:** LSP + Visual Intelligence + Voice + TUI enhancements PRD (§4.16–4.17, §4.26, §6 Enhancement).
 
- [ ] Voice-to-Code: `cantrik listen` via Whisper lokal (Ollama) — opt-in (§4.26 PRD)
- [ ] TTS notifikasi untuk background task — opt-in
- [ ] `/visualize [callgraph|architecture|dependencies]` → Mermaid/PlantUML di TUI atau export file (§4.17 PRD)
- [ ] LSP server mode — Cantrik sebagai Language Server untuk Neovim / VS Code / Helix (§4.16 PRD)
- [ ] TUI Split Pane: thinking log | code preview | semantic diff | approval panel (§6 Enhancement PRD)
- [ ] Cultural Wisdom Mode: `cultural_wisdom = "light"` / `"full"` (§6 Enhancement PRD)
- [ ] Multi-root Workspace: support monorepo / beberapa project folder
 
**Definition of Done:** Minimal satu alur voice atau visual atau LSP teruji end-to-end; cultural wisdom mode bisa dikonfigurasi.
 
---
 
## Sprint 19 — Ekosistem & distribusi (Phase 4)
 
**Goal:** Phase 4 — Ecosystem PRD.
 
- [ ] Hub/website `cantrik.dev` (placeholder dokumentasi + registry plugin)
- [ ] `cantrik init --template <name>` — bootstrap project dengan template per framework
- [ ] Air-gapped / enterprise offline mode — 100% lokal, tanpa cloud sama sekali
- [ ] Packaging: Homebrew, deb/apt, pacman, Nix flake, winget (bertahap)
- [ ] VS Code extension — side panel expose Cantrik capabilities
- [ ] Desktop companion app (Tauri) — monitor daemon + notifikasi; scope rilis awal terbatas
- [ ] Tech Debt Scanner production-ready: `/health` — outdated deps, CVE, test coverage, clippy (§4.14 PRD)
- [ ] Adaptive Begawan Style Learning — belajar dari history approval, simpan ke Tier 4 Global Memory (§4.15 PRD)
 
**Definition of Done:** Rilis alpha publik + dokumentasi kontribusi + salah satu saluran distribusi utama aktif.
 
---
 
## Backlog — Phase 5 & eksplorasi
 
**Goal:** Phase 5 — Maturity & Excellence PRD.
 
- [ ] Full autonomous SWE-agent mode — end-to-end fix GitHub issues dengan high reliability
- [ ] Agent harness improvements: self-reflection loops, better re-planning, visibility dashboard
- [ ] Self-improvement: Cantrik menganalisis dan suggest improvement ke codebase Cantrik sendiri
- [ ] Benchmark formal vs SWE-bench / Terminal-Bench
- [ ] Community-driven recipes & templates di cantrik.dev
- [ ] Hybrid cloud execution: opt-in via SSH ke instance sendiri untuk task berat
- [ ] Perluasan bahasa tree-sitter: Kotlin, Swift, Dart, Zig, dll.
- [ ] TUI Split Pane jika belum selesai di Sprint 18
- [ ] `cantrik fix <issue-url>` full implementation jika di-defer di Sprint 16
- [ ] gVisor / Firecracker sandbox untuk isolasi enterprise-grade
 
---
 
## Catatan Operasional
 
- Update status tiap PR: `[ ]` → `[/]` → `[x]`.
- Satu sprint boleh beberapa PR kecil.
- Jika scope sprint meleset >30%, pindahkan item ke sprint berikutnya dan catat alasan singkat di PR atau di bawah item bersangkutan.
- Semua fitur baru yang tidak ada di sprint aktif → tambahkan ke Backlog dulu, baru triase ke sprint yang tepat.
- File PRD acuan: `prd/cantrik-prd.md` (bukan lagi `prd/cantrik-doc.js`)