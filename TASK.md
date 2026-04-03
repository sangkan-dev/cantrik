# TASK.md - Cantrik Sprint Board

Dokumen ini dipakai untuk tracking implementasi berdasarkan **PRD** di `prd/cantrik-doc.js` (bagian *Roadmap Pengembangan*, fitur inti, arsitektur). Satu sprint diasumsikan ~2 minggu.

## Legend

- `[ ]` belum dikerjakan
- `[/]` sedang berjalan
- `[x]` selesai

## Pemetaan PRD → Sprint (ringkas)

| Fase PRD (`cantrik-doc.js`) | Sasaran | Sprint di board |
|----------------------------|---------|-----------------|
| **Phase 0** — Fondasi (Bulan 1–2) | CLI dasar, bridge LLM, REPL awal, config | 1–4 |
| **Phase 1** — Core Intelligence (Bulan 3–4) | AST + vektor + memori sesi + alat file | 5–7 |
| **Phase 2** — Agentic (Bulan 5–6) | Tool eksekusi, checkpoint/audit, re-plan, multi-agent | 8–11 |
| **Phase 3** — Advanced (Bulan 7–9) | Daemon, plugin, routing biaya, MCP, diff semantik, kolaborasi, suara, Git/PR, web, LSP/visual, macro/rules | 12–17 |
| **Phase 4** — Ekosistem (Bulan 10–12) | Hub, template, air-gap, distribusi, VS Code, Tauri, tech debt & adaptive learning | 18 |
| **Phase 5** — Maturity | Mode SWE otonom penuh, self-improve, benchmark | Backlog |

## Baseline (Sudah Ada)

- [x] Inisialisasi project Rust (`Cargo.toml`, `crates/cantrik-cli/src/main.rs`)
- [x] Draft PRD tersedia di folder `prd/` (`cantrik-doc.js`)

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

- [ ] Integrasi `ratatui` + `crossterm`
- [ ] Render *thinking log* + output streaming (sesuai gaya *Terminal UX* PRD)
- [ ] Riwayat input + state sesi in-memory
- [ ] Perintah built-in minimal: `/cost`, `/memory`, `/doctor` (sesuai tabel *Built-in Commands* PRD)

**Definition of Done:** REPL bisa sesi percakapan singkat dengan log dan tiga perintah di atas.

---

## Sprint 5 — Codebase intelligence: AST & indexing (Phase 1)

**Goal:** Pemahaman struktur kode selaras *Codebase Intelligence* PRD.

- [ ] Integrasi `tree-sitter` — prioritas PRD Phase 1: Rust, Python, JS/TS, Go (perluasan bahasa di PRD: Java, C/C++, PHP, Ruby, SQL, TOML, JSON, YAML, Markdown = backlog bertahap)
- [ ] AST-aware chunking (batas fungsi/class, bukan potong karakter naif)
- [ ] *Dependency graph* (siapa memanggil siapa) — sesuai fitur inti PRD
- [ ] File scanner `.gitignore`-aware + batas ukuran/file biner
- [ ] Re-index inkremental (hanya file berubah)

**Definition of Done:** Index folder proyek menghasilkan chunk AST + metadata path/symbol; scan menghormati `.gitignore`.

---

## Sprint 6 — Vector store & pencarian semantik (Phase 1)

**Goal:** *Tier 3 Project Memory* PRD — LanceDB embedded, embedding lokal.

- [ ] Integrasi LanceDB (embedded) di `.cantrik/index/` (selaras *Directory Structure* PRD)
- [ ] Pipeline embedding default Ollama (`nomic-embed-text`); opsi cloud/ model lain sesuai PRD
- [ ] Metadata chunk (path, symbol, bahasa)
- [ ] Semantic search: API internal + perintah CLI (`index` / search terkait)

**Definition of Done:** Query teks mengembalikan chunk relevan dari index lokal tanpa kirim kode ke cloud hanya untuk embedding default.

---

## Sprint 7 — Session memory & alat file (Phase 1)

**Goal:** *Session Memory* + *File tools* PRD (SQLite/sqlx, ringkasan, pruning, anchors).

- [ ] SQLite untuk histori sesi + ringkasan (path PRD: `~/.local/share/cantrik/` ↔ konvensi final di implementasi)
- [ ] Simpan keputusan penting per sesi
- [ ] Context pruning + summarization saat window penuh (*Context Compression* PRD)
- [ ] *Memory anchors* (`anchors.md` global + opsi proyek)
- [ ] Tool: `read_file`, `write_file` dengan **diff preview** + approval sebelum tulis

**Definition of Done:** Sesi bisa dilanjutkan dengan ringkasan; tulis file tidak tanpa preview/approve; anchor ikut dimuat ke konteks.

---

## Sprint 8 — Tool system & sandbox (Phase 2)

**Goal:** Eksekusi aman — selaras *Sandboxed Execution* + *Permission Tiers* PRD.

- [ ] Registry tool: `run_command`, `search`/grep codebase, `read_file`/`write_file` (integrasi penuh dengan tier)
- [ ] Tier: forbidden / require_approval / auto_approve
- [ ] Prompt approval untuk write, exec, network
- [ ] Sandbox level `restricted` minimum viable (bubblewrap Linux / setara macOS sesuai PRD)
- [ ] `git_ops` read-only + `web_fetch` opsional dengan approval (sesuai Phase 2 PRD)

**Definition of Done:** Tidak ada write/exec/network tanpa jalur approval; sandbox default aktif untuk exec.

---

## Sprint 9 — Checkpoint, rollback, audit (Phase 2)

**Goal:** *Checkpointing & Rollback* + *Audit Log* PRD.

- [ ] Auto checkpoint sebelum operasi write (`.cantrik/checkpoints/`)
- [ ] Perintah `rollback` + list checkpoint
- [ ] Audit log append-only (+ human-readable sesuai contoh PRD)
- [ ] Cost tracking dasar per aksi / model

**Definition of Done:** Satu alur tulis file bisa di-rollback; aksi tercatat di audit.

---

## Sprint 10 — Planning, re-planning & escalation (Phase 2)

**Goal:** *Long-horizon Planning* + *Stuck Detection* PRD.

- [ ] Mesin plan → act → evaluate; re-plan jika langkah gagal
- [ ] Deteksi stuck (threshold percobaan, contoh PRD: 3)
- [ ] Eskalasi ke user dengan ringkasan percobaan
- [ ] Integrasi ke mode `--plan` / `plan` subcommand

**Definition of Done:** Task multi-step percobaan sederhana bisa re-plan atau berhenti dengan pesan eskalasi jelas.

---

## Sprint 11 — Multi-agent v1 (Phase 2)

**Goal:** *Multi-Agent Orchestration* PRD.

- [ ] Orchestrator + konteks sub-agent terpisah
- [ ] Eksekusi paralel (`tokio`)
- [ ] Summary propagation ke orchestrator
- [ ] Batas kedalaman spawn (default 3)
- [ ] Isolasi kegagalan satu sub-agent

**Definition of Done:** Satu task terdekomposisi ke beberapa sub-agent paralel lebih cepat dari urutan serial pada skenario uji.

---

## Sprint 12 — Background agent & daemon (Phase 3)

**Goal:** *Background Agent Mode* PRD (daemon, persistensi, notifikasi).

- [ ] Mode background / long-running + persist progress (SQLite)
- [ ] Integrasi daemon: systemd user (Linux) / launchd (macOS) — sesuai kemampuan rilis
- [ ] Notifikasi saat perlu approval (desktop / webhook / flag poll)

**Definition of Done:** Task panjang tetap berjalan setelah terminal tertutup pada skenario yang didukung.

---

## Sprint 13 — Plugin & skill system (Phase 3)

**Goal:** Tiga lapis PRD — skill `.md`, Lua `mlua`, WASM `wasmtime`.

- [ ] Auto-inject skill (`.cantrik/skills/*.md`)
- [ ] Runtime Lua untuk plugin proyek
- [ ] Runtime WASM untuk plugin advanced
- [ ] Perintah install/list/update (registry lokal dulu; *cantrik.dev* di Phase 4)

**Definition of Done:** Minimal satu contoh plugin Lua dan satu WASM berjalan di lingkungan dev.

---

## Sprint 14 — Smart routing, biaya & MCP (Phase 3)

**Goal:** *Smart Routing & Cost Control* + *MCP Integration* PRD.

- [ ] Routing model otomatis / threshold (opsi `providers.toml` / config)
- [ ] Anggaran biaya per sesi & per bulan (config)
- [ ] `cantrik serve --mcp` (server MCP)
- [ ] Konsumsi MCP server eksternal (client)

**Definition of Done:** Cantrik bisa dipanggil dari host MCP dan memanggil tools MCP lain pada skenario uji.

---

## Sprint 15 — Semantic diff & kolaborasi (Phase 3)

**Goal:** *Semantic Diff & Merge* + kolaborasi PRD.

- [ ] Output semantic diff + risk assessment + fungsi terdampak
- [ ] Cek cakupan tes / saran (minimal heuristik)
- [ ] Mode kolaboratif: export/import konteks atau berbagi sesi (sesuai PRD)

**Definition of Done:** Pengguna bisa meninjau ringkasan perubahan semantik sebelum apply.

---

## Sprint 16 — Git-native workflow, provenance & web (Phase 3)

**Goal:** *Deep Git-Native Workflow* + *Provenance* + *Web Research* PRD.

- [ ] Auto-branch per task; pesan commit berbasis ringkasan
- [ ] `pr create` / integrasi GitHub atau GitLab (minimal satu penyedia)
- [ ] Deteksi konflik + saran resolusi dasar
- [ ] Mode `fix <issue-url>` (stretch — boleh defer dengan catatan)
- [ ] Metadata provenance / explainability pada diff atau komentar (selaras contoh PRD)
- [ ] Web research / browse dengan approval eksplisit

**Definition of Done:** Alur lokal dari branch hingga PR dapat diotomatisasi pada repo demo; aksi web hanya setelah approve.

---

## Sprint 17 — Suara, visual, LSP, macro & rules (Phase 3)

**Goal:** Item sisa Phase 3 PRD.

- [ ] Voice-to-code (Whisper lokal) + TTS notifikasi (opsional per platform)
- [ ] `/visualize` — Mermaid / PlantUML di TUI
- [ ] Integrasi LSP (Neovim/VS Code) — scope minimal jelas di PR
- [ ] Macro & recipe system; `.cantrik/rules.md` (*Custom Guardrails* PRD)

**Definition of Done:** Minimal satu alur suara atau visual atau rules teruji end-to-end (tidak perlu semua platform sekaligus).

---

## Sprint 18 — Ekosistem & distribusi (Phase 4)

**Goal:** *Phase 4 — Ecosystem* PRD.

- [ ] Hub/website `cantrik.dev` (placeholder dokumentasi + registry)
- [ ] `cantrik init` templates per framework
- [ ] Air-gapped / enterprise offline mode
- [ ] Packaging: Homebrew, deb/apt, pacman, Nix, winget (bertahap)
- [ ] VS Code extension (scope panel)
- [ ] Desktop companion (Tauri) — scope rilis awal
- [ ] *Tech debt scanner* (`/health`) + *Adaptive Begawan Style Learning* (PRD)

**Definition of Done:** Rilis alpha publik + dokumentasi kontribusi + salah satu saluran distribusi utama.

---

## Backlog — Phase 5 & eksplorasi

**Goal:** *Phase 5 — Maturity & Excellence* PRD.

- [ ] Full autonomous SWE-agent mode
- [ ] Self-improvement loop pada codebase Cantrik
- [ ] Benchmark formal vs SWE-bench
- [ ] Perluasan provider/matrix (OpenRouter, Groq, Azure, …) jika belum
- [ ] Perluasan bahasa tree-sitter penuh sesuai daftar PRD
- [ ] *Enhancement* PRD: TUI split pane, multi-root workspace, cultural wisdom mode

## Catatan operasional

- Update status tiap PR: `[ ]` → `[/]` → `[x]`.
- Satu sprint boleh beberapa PR kecil.
- Jika scope sprint meleset >30%, pindahkan item ke sprint berikutnya dan catat alasan singkat di PR atau di bawah item bersangkutan.
