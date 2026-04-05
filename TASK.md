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

**Catatan:** `/memory` menjelaskan tier DB + anchors; pelacakan biaya memakai `/cost` atau `cantrik cost` (Sprint 14). `/doctor` memakai `doctor::report_lines` yang sama dengan subcommand `cantrik doctor`.

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
 
- [x] Orchestrator + konteks sub-agent terpisah (isolated context window)
- [x] Eksekusi paralel via `tokio`
- [x] Summary propagation ke orchestrator (hemat token)
- [x] Batas kedalaman spawn (default: 3)
- [x] Failure isolation — satu sub-agent gagal tidak stop yang lain
- [x] Structured Plan & Act Mode — stub awal: Planner (read-only) + Builder (approval) (§4.12 PRD)
 
**Definition of Done:** Task terdekomposisi ke beberapa sub-agent paralel; Planner dapat berjalan tanpa akses write.
 
**Batas MVP Sprint 11:** Sub-agent memakai `llm::ask_complete_text` (ephemeral, tanpa `append_message` ke SQLite); rekursi sub-agent → sub-agent belum diimplementasikan (hanya API `depth` + penolakan jika `depth >= max_spawn_depth`); Reviewer §4.12 ditunda; Builder = stub teks LLM (bukan eksekusi tool otomatis).
 
---
 
## Sprint 12 — Background agent & daemon (Phase 3)
 
**Goal:** Background Agent Mode PRD (§4.3).
 
- [x] Mode background / long-running + persist progress ke SQLite (`background_jobs`, `cantrik background`, `cantrik daemon`)
- [x] Integrasi daemon: contoh systemd user unit (`contrib/systemd/cantrik-daemon.service`) dan launchd (`contrib/launchd/com.cantrik.daemon.plist`)
- [x] Notifikasi saat perlu approval: desktop (`notify-send` / `osascript`), webhook `[background].webhook_url`, file flag (default `~/.local/share/cantrik/approval-pending.flag`)
- [x] `cantrik status` — cek progress task background (`--all` = semua proyek di DB)
 
**Definition of Done:** Task panjang tetap berjalan setelah terminal tertutup; notifikasi terkirim saat approval dibutuhkan.

**Batas MVP (Sprint 12):** Runner daemon memanggil satu putaran `complete_with_session` per siklus klaim job; setelah setiap putaran (jika belum mencapai `[background].max_llm_rounds`, default **2**) job masuk `waiting_approval` dan user melanjutkan dengan `cantrik background resume <id>`. Set `max_llm_rounds = 1` untuk satu putaran lalu `completed` tanpa jeda approval. Gate “approval” sebelum tool tulis penuh belum diintegrasikan ke orkestrator multi-tool — hanya jeda antar putaran LLM. Checkpoint per langkah di filesystem tidak wajib; heartbeat + state ada di SQLite.
 
---
 
## Sprint 13 — Plugin & skill system (Phase 3)
 
**Goal:** Tiga lapis PRD — skill `.md`, Lua `mlua`, WASM `wasmtime` (§7 PRD).
 
- [x] Auto-inject skill (`.cantrik/skills/*.md`) berdasarkan skor keyword / nama file (relevansi MVP); `[skills]` di config (`auto_inject`, `max_total_chars`, `max_files`, `files`)
- [x] Auto-inject `.cantrik/rules.md` — selalu disisipkan di `build_llm_prompt` (kecuali `CANTRIK_NO_RULES`) — §4.19 PRD
- [x] Runtime Lua (`mlua`) — `.cantrik/plugins/*.lua`, host `cantrik.suggest` / `log` / `warn` / `require_approval` (stub log); hook `on_task_start` (CLI `ask`), `after_write` (setelah `tool_write_file` sukses)
- [x] Runtime WASM (`wasmtime`) — `.cantrik/plugins/*.wasm` tanpa import; panggil export `after_write_ping` jika ada (contoh WAT: `contrib/wasm/after_write_ping.wat`)
- [x] Perintah `cantrik skill install/list/update/remove` — registry lokal `~/.local/share/cantrik/skill-registry/<name>/` + `manifest.toml`; state `.cantrik/installed-skills.toml`
- [x] Macro (§4.18): `cantrik macro record` / `macro add -- …` / `macro stop` / `macro run` / `macro list` — file JSON di `.cantrik/macros/`
 
**Definition of Done:** Minimal satu contoh plugin Lua dan satu WASM berjalan; rules.md selalu di-inject; satu macro bisa di-record dan di-replay.

**Batas MVP (Sprint 13):** Relevansi skill hanya heuristik token (bukan embedding). Registry skill hanya lokal (bukan cantrik.dev). WASM tidak menerima path file di guest (hanya hook `after_write_ping` tanpa argumen); tidak ada WASI / akses FS host dari WASM. `cantrik.require_approval` di Lua hanya log — belum terhubung ke pipeline approval guardrails. `on_task_start` hanya dijalankan dari `cantrik ask` (bukan REPL/agents semua jalur). Macro: langkah direkam per `macro add`, bukan auto-hook shell.
 
---
 
## Sprint 14 — Smart routing, biaya & MCP (Phase 3)
 
**Goal:** Smart Routing + Cost Control + MCP Integration PRD (§3, §4.9).
 
- [x] Routing model otomatis berdasarkan task complexity (simple/medium/complex threshold)
- [x] Budget: `max_cost_per_session` dan `max_cost_per_month` dari config
- [x] `/cost` command — tampilkan usage & biaya real per session + bulan ini
- [x] `cantrik serve --mcp` — Cantrik sebagai MCP server
- [x] Konsumsi MCP server eksternal (GitHub MCP, Postgres MCP, dll.) sebagai client
 
**Definition of Done:** Cantrik bisa dipanggil dari host MCP dan memanggil tools MCP lain; routing model berfungsi sesuai threshold.

**Batas MVP Sprint 14:** Biaya = **perkiraan** dari panjang UTF-8 + tabel harga statis per provider/model (`llm/cost.rs`); token nyata dari API belum dipakai. `auto_route` mengganti **target pertama** rantai LLM bila `[routing].auto_route` + `[routing.thresholds]` ada dan `routing_prompt` diset (REPL/`ask` memakai teks user; ringkasan internal memakai `routing_prompt: None`). Budget melebihi cap → error `LlmError::BudgetExceeded` (bukan fallback otomatis). MCP: crate **`rmcp` 1.3** (stdio server + child-process client); tool server `cantrik_ask`; client CLI `cantrik mcp call <server> <tool> --json '{}'`. Registrasi tool MCP di `tool_system` / resources penuh → sprint berikutnya.
 
---
 
## Sprint 15 — Semantic diff & kolaborasi (Phase 3)
 
**Goal:** Semantic Diff & Merge + Collaborative Mode PRD (§4.8, §4.23).
 
- [x] Output semantic diff + risk assessment + fungsi/file terdampak (`cantrik diff`; overlay dari `.cantrik/index/ast/` bila ada)
- [x] Cek cakupan tes per perubahan — saran minimal heuristik (`tests_hint` + pesan di `cantrik diff`)
- [x] Conflict detection Git + saran resolusi (`cantrik diff --conflicts` + `git status --porcelain` / petunjuk marker)
- [x] Export/import context (`cantrik export`, `cantrik import` — bundle JSON skema v1)
- [x] Context Handoff Protocol: `cantrik handoff` → `.cantrik/handoff-YYYY-MM-DD.md` (UTC) (§4.23 PRD)
- [x] Session Replay: JSON log + timeline stdout (`cantrik replay export`, `cantrik replay play`) (§4.27 PRD)
 
**Batas MVP (Sprint 15):** Tanpa re-eksekusi tool/agent; tanpa call graph lintas file; tanpa resolusi konflik LLM/merge otomatis; tanpa TUI split-pane. Konfigurasi opsional `[collab]` di `cantrik.toml`: `max_files_in_report`, `replay_tail_messages`.
 
**Definition of Done:** User bisa review ringkasan perubahan semantik sebelum apply; handoff file bisa di-generate dan di-load.
 
---
 
## Sprint 16 — Git-native workflow, review & web research (Phase 3)
 
**Goal:** Deep Git-Native Workflow + `cantrik review` + Web Research PRD (§4.11, §4.13, §4.22).
 
- [x] Auto-branch per task: `cantrik workspace branch start <slug>` → `feature/cantrik-<slug>` (prefix dari `[git_workflow].branch_prefix`)
- [x] AI-generated commit message + approval: `cantrik workspace commit` (LLM dari `git diff --cached`); `git commit` hanya dengan `--approve`
- [x] `cantrik pr create` — GitHub via `gh pr create` (origin harus GitHub); `[git_workflow].pr_provider = "none"` mematikan
- [x] `cantrik fix <url>` — stub MVP + langkah manual (`fetch` / `agents` / `workspace commit` / `pr create`); loop SWE penuh ditunda
- [x] `cantrik review` — LLM pada diff ter-staging (default) atau `--worktree`; `--soft` untuk hook; contoh hook: [contrib/git-hooks/pre-commit-review.sample](contrib/git-hooks/pre-commit-review.sample)
- [x] Web research: tool `web_search` / `browse_page` / `fetch_docs` (guardrails); CLI `cantrik web search|fetch` dengan `--approve` (§4.13)
 
**Batas MVP:** Tanpa browser/JS sandbox penuh; pencarian via DuckDuckGo HTML (parsing rapuh); tanpa GitLab/Bitbucket native; `cantrik fix` tanpa otomasi test+PR. Konfigurasi opsional `[git_workflow]` di `cantrik.toml`.
 
**Definition of Done:** Alur lokal dari auto-branch hingga PR dapat diotomatisasi pada repo demo; review command bisa run standalone; web fetch hanya setelah approve.
 
---
 
## Sprint 17 — Intelligence tools: explain, teach, dependency, experiment (Phase 3)
 
**Goal:** Code archaeology, knowledge extraction, dependency intel, experiment mode PRD (§4.20–4.25).
 
- [x] `cantrik explain [file] --why` — Code Archaeology via git blame + commit history (§4.20 PRD)
- [x] `cantrik teach` — generate ARCHITECTURE.md, ADR, API docs dari codebase (§4.25 PRD)
- [x] `cantrik teach --format wiki` — export ke format Obsidian/Notion/Confluence-compatible
- [x] `cantrik why <dep>`, `cantrik upgrade`, `cantrik audit` — Dependency Intelligence (§4.24 PRD)
- [x] Experiment mode: revert otomatis jika tes/write gagal (sudah ada; §4.21 PRD) — bandingkan benchmark sebelum/sesudah **ditunda** (fase 2)
 
**Batas MVP (Sprint 17):** Tanpa PR otomatis untuk explain; tanpa `cargo update` otomatis di `upgrade` (hanya saran LLM + konteks lock/tree); jika `cargo-audit` tidak terpasang, pesan jelas + `[intelligence].audit_command` opsional. Konfigurasi opsional `[intelligence]`: `explain_max_blame_lines`, `teach_max_files_scanned`, `audit_command`.
 
**Definition of Done:** Minimal `cantrik explain` dan `cantrik audit` berjalan end-to-end; experiment mode bisa revert otomatis berdasarkan hasil test.
 
---
 
## Sprint 18 — LSP, visual, voice & advanced UX (Phase 3)
 
**Goal:** LSP + Visual Intelligence + Voice + TUI enhancements PRD (§4.16–4.17, §4.26, §6 Enhancement).
 
- [x] Voice-to-Code: `cantrik listen` — opt-in `[ui] voice_enabled`; audio → Ollama `/api/transcribe` bila tersedia; `--raw-text` untuk uji tanpa audio (§4.26 PRD)
- [x] TTS notifikasi untuk background task — opt-in (`voice_enabled` + `espeak` / `say` pada Linux/macOS)
- [x] `/visualize` + `cantrik visualize` → Mermaid (callgraph dari indeks, architecture dari tree dir, dependencies dari `cargo tree`); export `--output` (§4.17 PRD)
- [x] LSP server mode (`cantrik lsp`) — stdio MVP: `documentSymbol` + hover dari `.cantrik/index/ast/chunks.jsonl` (subset PRD §4.16)
- [x] TUI split pane — `[ui] tui_split_pane`: assistant + panel preview (`/visualize`); panel “semantic diff / approval” penuh ditunda (§6 Enhancement PRD)
- [x] Cultural Wisdom Mode: `[ui] cultural_wisdom = "off" | "light" | "full"` — injeksi ke `build_llm_prompt` + REPL tanpa sesi (§6 Enhancement PRD)
- [x] Multi-root workspace (MVP) — `[workspace].extra_roots` menggabungkan fingerprint **sesi** (`session_project_fingerprint`); indeks multi-root otomatis belum
 
**Batas MVP (Sprint 18):** Tanpa PlantUML generator; **LSP:** stdio saja, tanpa completion/rename/diagnostics bahasa asli; simbol hanya dari indeks AST (`chunks.jsonl`); satu root per proses LSP (folder `initialize` / workspace folder pertama). **Multi-root:** hanya identitas sesi + usage yang mengikuti fingerprint gabungan; `cantrik index` / background jobs tetap per cwd utama. Callgraph hanya intra-file dari `graph.json`; STT bergantung build Ollama yang mendukung `/api/transcribe` + model whisper. Konfigurasi `[ui]`: `cultural_wisdom`, `voice_enabled`, `tui_split_pane`, `transcription_model`. Konfigurasi `[workspace]`: `extra_roots`.
 
**Definition of Done:** Minimal satu alur voice atau visual atau LSP teruji end-to-end; cultural wisdom mode bisa dikonfigurasi.
 
---
 
## Sprint 19 — Ekosistem & distribusi (Phase 4)
 
**Goal:** Phase 4 — Ecosystem PRD.
 
- [x] Hub / website — monorepo [`apps/cantrik-site`](apps/cantrik-site/) (SvelteKit static, nuansa Sangkan); target deploy `cantrik.sangkan.dev`; registry plugin = JSON statis MVP
- [x] `cantrik init --template <name>` — MVP: `generic`, `rust-cli` (`.cantrik/cantrik.toml` + `rules.md`); template per framework penuh ditunda
- [x] Saluran distribusi utama (MVP) — binary Linux via **GitHub Releases** pada tag `v*` (`.github/workflows/release.yml`); Homebrew/deb/Nix/winget = lanjutan
- [x] Air-gapped / enterprise offline mode — MVP: `[llm] offline` + `CANTRIK_OFFLINE`; rantai LLM hanya Ollama loopback; fitur lain tetap bisa pakai jaringan (terdokumentasi)
- [x] Packaging tambahan (MVP): formula Homebrew + nfpm `.deb`; pacman / Nix / winget menyusul
- [x] VS Code extension — palette + output channel + LSP stdio opsional ([`apps/cantrik-vscode`](apps/cantrik-vscode/))
- [x] Desktop companion — polling flag approval + notifikasi ([`apps/cantrik-tray`](apps/cantrik-tray/)); shell Tauri penuh ditunda
- [x] Tech Debt Scanner v0: `cantrik health` + `/health` di REPL (audit, clippy, test, timeout; bukan pengganti pipeline CI penuh)
- [x] Adaptive Begawan MVP — tabel `approval_memory`, rekam `--approve` (file/exec/experiment), injeksi prompt + toggle `[memory] adaptive_begawan`
 
**Batas MVP (Sprint 19):** Hub = landing + nav docs/registry; plugin list = `static/registry/plugins.json`; CI terpisah untuk site; tidak ada marketplace atau auth. Init = 2 template saja. Rilis = satu artefak `cantrik` (Linux) per tag; verifikasi checksum manual sampai ada signing otomatis.
 
**Definition of Done:** Rilis alpha publik + dokumentasi kontribusi + salah satu saluran distribusi utama aktif.
 
---
 
## Backlog — Phase 4 lanjutan & Phase 5
 
**Phase 4 (tunda / lanjutan):** audit jaringan menyeluruh untuk enterprise; pacman / Nix / winget; side panel VS Code kaya fitur; Tauri tray UI; cakupan `/health` (coverage, outdated tree) diperdalam.
 
**Foundation (iterasi backlog, bukan penutup item):** tabel *Network surfaces* + blok HTTP saat offline; `cantrik health --tree` / `--outdated` / `--coverage` / `--deny` (opsional); artefak [`packaging/arch`](packaging/arch/PKGBUILD), [`packaging/nix`](packaging/nix/flake.nix), [`packaging/winget`](packaging/winget/Sangkan.Cantrik.yaml); panel aktivitas VS Code + status bar + webview status JSON; [`apps/cantrik-tauri`](apps/cantrik-tauri/) (Tauri v2 tray buildable); `cantrik fix --approve --fetch` dan rantai `--run-agents`; `cantrik status --json`; `cantrik agents --reflect`; halaman [`/dashboard`](apps/cantrik-site/src/routes/dashboard/+page.svelte); [`static/registry/recipes.json`](apps/cantrik-site/static/registry/recipes.json); [`scripts/phase5-smoke.sh`](scripts/phase5-smoke.sh) + [`scripts/swe-fix-demo.sh`](scripts/swe-fix-demo.sh).
 
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