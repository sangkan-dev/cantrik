# ꦕꦤ꧀ꦠꦿꦶꦏ꧀ — CANTRIK

> **Open-Source AI CLI Agent**
> Product Requirements Document · Technical Specification · Roadmap
> Version 1.1 · 2025

---

## Daftar Isi

1. [Filosofi & Visi](#1-filosofi--visi)
2. [Arsitektur Sistem](#2-arsitektur-sistem)
3. [LLM Bridge — Multi-Provider System](#3-llm-bridge--multi-provider-system)
4. [Fitur Inti](#4-fitur-inti)
5. [Guardrails & Sistem Keamanan](#5-guardrails--sistem-keamanan)
6. [Terminal UX — Ancient Cybernetics CLI](#6-terminal-ux--ancient-cybernetics-cli)
7. [Plugin & Skill System](#7-plugin--skill-system)
8. [Roadmap Pengembangan](#8-roadmap-pengembangan)
9. [Referensi Konfigurasi](#9-referensi-konfigurasi)
10. [Open Source & Kontribusi](#10-open-source--kontribusi)

---

## 1. Filosofi & Visi

Dalam pewayangan dan budaya Jawa kuno, **Cantrik** adalah seorang murid sekaligus asisten setia dari seorang **Begawan** — guru besar atau pertapa yang bijaksana. Seorang Cantrik bertugas menyiapkan segala keperluan teknis, membersihkan jalan, dan membantu sang guru mewujudkan pemikirannya *tanpa pernah melampaui kehendak gurunya.*

### Makna Teknis

Di terminal, Cantrik adalah CLI Agent yang menyiapkan boilerplate, membaca konteks kode, menjalankan script, dan mencari bug — sehingga **kamu (Sang Begawan)** bisa fokus pada arsitektur tingkat tinggi.

### Tiga Pilar Utama

- **Etika Tinggi.** Cantrik tidak pernah bertindak tanpa persetujuan Begawan. Setiap aksi yang memodifikasi sistem memerlukan approval eksplisit.
- **Bare-metal Fast.** Dibangun di atas Rust dengan async tokio — tidak ada lag, tidak ada overhead, tidak ada runtime berat.
- **100% Open Source.** Cantrik bukan wrapper API biasa. Setiap baris kodenya terbuka, dapat diaudit, dan dapat dikontribusi.

### Perbedaan dengan Tools Lain

| Aspek | Claude Code | Gemini CLI | Cantrik |
|---|---|---|---|
| Source | Closed (leaked) | Open Source | **Open Source ✓** |
| Core Language | TypeScript | TypeScript | **Rust ✓** |
| Multi-provider LLM | Claude only | Gemini only | **Plug-and-play ✓** |
| Vector Memory | Limited | Limited | **Native ✓** |
| Plugin System | Partial | Partial | **Lua + WASM ✓** |
| Multi-Agent | In Dev | Limited | **Native ✓** |
| Offline Support | No | Partial | **Full (Ollama) ✓** |
| Git-Native Workflow | Partial | No | **Native ✓** |
| Adaptive Learning | No | No | **Native ✓** |

---

## 2. Arsitektur Sistem

### Stack Teknologi

| Layer | Teknologi | Alasan |
|---|---|---|
| Core Engine | `Rust + tokio` | Async, memory-safe, zero-cost abstraction |
| CLI Framework | `clap v4` | Mature, derive macro, shell completion |
| Terminal UI | `ratatui + crossterm` | Cross-platform, streaming support |
| Vector Store | `LanceDB (embedded)` | Native Rust, no server required |
| Relational DB | `SQLite via sqlx` | Session memory, audit log, config |
| AST Parsing | `tree-sitter` | Multi-language, incremental parsing |
| Plugin System | `mlua + wasmtime` | Lua ringan, WASM untuk plugin advanced |
| HTTP Client | `reqwest + async` | Streaming LLM response support |
| Serialization | `serde + serde_json` | De-facto standard Rust ecosystem |

### Memory Architecture — 4 Tiers

Cantrik menggunakan sistem memori berlapis yang terinspirasi dari cara kerja memori manusia:

| Tier | Nama | Storage | Deskripsi |
|---|---|---|---|
| Tier 1 | Working Memory | Context Window | Percakapan aktif sesi ini |
| Tier 2 | Session Memory | SQLite per-folder | History sesi, keputusan, context ringkas |
| Tier 3 | Project Memory | LanceDB vector index | Index semantik seluruh codebase |
| Tier 4 | Global Memory | `~/.config/cantrik/` | Preferensi user, pola kerja, Begawan anchors |

### Directory Structure

```
~/.config/cantrik/
  config.toml              # Global user preferences
  providers.toml           # API keys & LLM provider config
  anchors.md               # Memory anchors (always in context)

~/.local/share/cantrik/
  memory.db                # SQLite: session history, audit log
  audit.log                # Human-readable action log

<project_root>/
  .cantrik/
    cantrik.toml           # Project-level config
    rules.md               # Custom guardrails — always injected
    index/                 # LanceDB vector index
    skills/
      backend.md           # Context: arsitektur backend
      database.md          # Context: skema & konvensi DB
      deploy.md            # Context: cara deploy proyek
    sessions/              # Session replay files
    checkpoints/           # Rollback snapshots
    plugins/               # Project-specific Lua plugins
```

### Crate Structure

```
cantrik/
├── crates/
│   ├── cantrik-core/      # Core agent logic, context management
│   ├── cantrik-llm/       # LLM bridge & provider implementations
│   ├── cantrik-rag/       # Vector store, AST parsing, indexing
│   ├── cantrik-tools/     # Tool system, file ops, command exec
│   ├── cantrik-tui/       # Terminal UI dengan ratatui
│   ├── cantrik-plugins/   # Lua + WASM plugin runtime
│   └── cantrik-mcp/       # MCP server & client
├── cantrik-cli/           # Binary entry point
├── docs/                  # Dokumentasi
├── plugins/               # Built-in plugins
└── templates/             # cantrik init templates
```

---

## 3. LLM Bridge — Multi-Provider System

Salah satu keunggulan utama Cantrik adalah sistem bridge modular yang memungkinkan plug-and-play provider LLM tanpa mengubah core logic.

### Provider Matrix

| Provider | Streaming | Vision | Tool Use | Embedding | Offline |
|---|---|---|---|---|---|
| Anthropic Claude | ✓ | ✓ | ✓ | — | — |
| Google Gemini | ✓ | ✓ | ✓ | ✓ | — |
| OpenAI / Azure | ✓ | ✓ | ✓ | ✓ | — |
| Ollama (Local) | ✓ | ✓* | ✓* | ✓ | **✓ FULL** |
| OpenRouter | ✓ | ✓* | ✓* | — | — |
| Groq | ✓ | — | ✓ | — | — |

> *\* Tergantung model yang digunakan*

### Smart Routing & Cost Control

Cantrik secara otomatis memilih model yang tepat berdasarkan kompleksitas task:

- **Task Ringan** (rename variable, boilerplate, format) → Model kecil/murah (Haiku, Flash, lokal)
- **Task Sedang** (debug, explain code, test writing) → Model menengah (Sonnet, Pro)
- **Task Berat** (arsitektur, threading, security audit) → Model besar (Opus, Ultra)

```toml
# providers.toml
[routing]
auto_route           = true
max_cost_per_session = 0.50   # USD
max_cost_per_month   = 10.00  # USD

[routing.thresholds]
simple  = "claude-haiku-4"
medium  = "claude-sonnet-4"
complex = "claude-opus-4"

[fallback]
chain = ["claude-sonnet-4", "gemini-flash", "ollama/llama3"]
```

### Embedding Strategy (Offline-First)

Indexing codebase dilakukan 100% lokal menggunakan model embedding via Ollama — tidak ada kode yang dikirim ke cloud hanya untuk indexing:

- `nomic-embed-text` — Default, ringan, akurat untuk kode
- `mxbai-embed-large` — Lebih akurat untuk proyek besar
- Cloud embedding (OpenAI, Gemini) tersedia sebagai opsi opsional

---

## 4. Fitur Inti

### 4.1 Codebase Intelligence (RAG Lokal)

Cantrik tidak hanya membaca file secara naif — dia memahami struktur kode secara semantik melalui kombinasi AST parsing dan vector search:

- **AST-aware chunking:** File dipotong berdasarkan boundary fungsi/class, bukan karakter — menghasilkan chunk yang semantically meaningful
- **Dependency graph:** Cantrik tahu fungsi mana yang memanggil fungsi mana — bisa menjawab *"Di mana `auth_middleware` dipakai?"*
- **Gitignore-aware:** Otomatis mengabaikan file di `.gitignore`, file biner, dan file di atas threshold ukuran
- **Incremental re-index:** Hanya file yang berubah yang di-index ulang, bukan seluruh codebase

Bahasa yang didukung melalui tree-sitter:
```
Rust, Python, JavaScript, TypeScript, Go, Java, C/C++, PHP, Ruby,
SQL, TOML, JSON, YAML, Markdown
```

### 4.2 Multi-Agent Orchestration

Cantrik mendukung spawning sub-agent secara paralel untuk task yang dapat di-decompose:

```
Cantrik Orchestrator
├── Sub-agent A: "Baca semua file auth/"        → paralel
├── Sub-agent B: "Cek test coverage module X"   → paralel
└── Sub-agent C: "Search bug di handler/"       → paralel
         ↓ semua selesai
   Orchestrator synthesize hasil → jawab user
```

- **Isolated context:** Setiap sub-agent punya context window terpisah
- **Summary propagation:** Sub-agent hanya kirim ringkasan ke orchestrator — hemat token
- **Depth limit:** Sub-agent bisa spawn sub-agent, tapi dengan batas kedalaman (default: 3)
- **Failure isolation:** Jika satu sub-agent gagal, yang lain tetap jalan

### 4.3 Background Agent Mode

Cantrik dapat berjalan sebagai daemon di background — kamu bisa tutup terminal dan lanjut kerja:

```bash
# Jalankan task panjang di background
cantrik background "refactor semua endpoint ke pattern baru" --notify

# Cek status
cantrik status

# Cantrik pause dan kirim notif jika butuh approval
# Bisa via: desktop notification / webhook / file flag
```

- **Daemon mode:** Jalan via systemd user service (Linux) atau launchd (macOS)
- **Checkpoint auto-save:** Progress tersimpan ke SQLite — tidak hilang jika mati mendadak
- **Notification channels:** Desktop (notify-send / osascript), webhook URL, atau file flag yang bisa di-poll

### 4.4 Long-horizon Planning dengan Re-planning

Untuk task kompleks, Cantrik tidak sekadar membuat plan linear — dia mengevaluasi hasil setiap step dan re-plan jika diperlukan:

```
1. Buat initial plan berdasarkan task
2. Eksekusi step 1
3. Evaluasi hasil: apakah sesuai ekspektasi?
   ├── Ya  → lanjut step berikutnya
   └── Tidak → RE-PLAN dengan informasi baru
4. Stuck detection setelah 3 kali gagal → minta bantuan user
```

Cantrik tahu kapan harus menyerah dan meminta bantuan Begawan — ini adalah fitur, bukan kelemahan.

### 4.5 Checkpointing & Rollback

Sebelum setiap operasi write, Cantrik otomatis membuat snapshot:

```
.cantrik/checkpoints/
  checkpoint-001-before-auth-refactor/
    src/auth/middleware.rs    # file asli
    src/handlers/login.rs     # file asli
    meta.json                 # timestamp, task description
```

```bash
cantrik rollback              # rollback ke checkpoint terakhir
cantrik rollback --list       # lihat semua checkpoint
cantrik rollback 001          # rollback ke checkpoint spesifik
```

### 4.6 Context Compression Cerdas

Saat context window mendekati batas, Cantrik melakukan hierarchical summarization secara otomatis:

- **Summarization:** Percakapan lama diringkas menjadi ~500 token, disimpan ke Session Memory
- **Hot context:** File yang sedang diedit dan error terakhir selalu dipertahankan
- **Memory Anchors:** Instruksi penting yang TIDAK pernah dihapus dari context

```markdown
# ~/.config/cantrik/anchors.md
- Selalu gunakan error handling pattern Result<T, E>
- Database schema ada di docs/schema.sql
- Jangan gunakan unwrap() di production code
- Naming convention: snake_case untuk Rust, camelCase untuk JS
```

### 4.7 Sandboxed Execution

| Level | Implementasi | Kapan Dipakai |
|---|---|---|
| `none` | Raw execution, no isolation | Developer percaya penuh pada Cantrik |
| `restricted` | bubblewrap (Linux) / sandbox-exec (macOS) | **Default** — blokir network, batasi fs |
| `container` | Docker container ringan | Proyek sensitif, code yang tidak dikenal |

### 4.8 Semantic Diff & Merge

Cantrik memahami intent dari perubahan, bukan hanya text diff:

```
  Cantrik ingin mengubah fungsi `validate_token`:

  SEMANTIC CHANGE : Menambahkan expiry check
  AFFECTED        : 3 fungsi yang memanggil validate_token
  RISK            : Low — backward compatible
  TEST COVERAGE   : Belum ada test untuk expiry case

  Saran: Tambahkan test dulu sebelum apply? [Y/n/e(dit)/v(iew diff)]
```

### 4.9 MCP Integration (Dua Arah)

- **Sebagai server:** Claude Desktop, Cursor, atau MCP-compatible tool bisa pakai Cantrik sebagai tool (`cantrik serve --mcp`)
- **Sebagai client:** Cantrik bisa memanggil MCP server lain (GitHub MCP, Postgres MCP, Browser MCP, dll.)

### 4.10 Provenance & Explainability

Setiap baris kode yang ditulis Cantrik memiliki metadata audit:

```rust
// [cantrik: 2025-07-01T14:23Z | model: claude-sonnet-4 | task: fix auth | conf: high]
if claims.exp < Utc::now().timestamp() {
    return Err(AuthError::TokenExpired);
}
```

Metadata bisa disimpan sebagai inline comment atau di `.cantrik/provenance.json` (dikonfigurasi).

### 4.11 Deep Git-Native Workflow

Cantrik dirancang sebagai "AI pair programmer" yang menghormati workflow Git secara native:

- **Auto branch creation** per task: `feature/cantrik-refactor-auth-xyz`
- **AI-generated commit message** yang deskriptif + semantic summary perubahan
- `cantrik commit "deskripsi task"` → commit otomatis dengan approval
- `cantrik pr create "implement fitur X"` → buat Pull Request ke GitHub/GitLab/Bitbucket (via `gh` CLI atau MCP)
- `cantrik fix <github-issue-url>` → mode SWE-agent: analisis issue, fix, test, buat PR
- **Conflict detection** + saran resolusi otomatis
- **Watch mode:** monitor perubahan Git dan auto-suggest improvement

### 4.12 Structured Plan & Act Mode

```bash
cantrik plan "refactor authentication"
```

Cantrik buat rencana terstruktur (step-by-step, estimasi risiko, cost, affected files). User approve plan dulu, baru masuk ke **Act Mode** untuk eksekusi.

**Dual-agent internal:**
- **Planner** — read-only analysis, tidak bisa modifikasi apa pun
- **Builder** — full access dengan approval system aktif
- **Reviewer** — sub-agent yang cek lint, test, security setelah setiap step

### 4.13 Web Research & Sandboxed Browser Tool

Tool dengan approval eksplisit untuk akses web:
- `web_search` — cari dokumentasi, issue serupa di GitHub, best practices
- `browse_page` — baca halaman URL spesifik
- `fetch_docs` — ambil API reference, crate docs, dll.
- Sandbox ketat — tidak boleh akses data sensitif tanpa izin user

### 4.14 Automated Tech Debt Scanner & Health Check

```bash
/health
cantrik doctor
```

Background periodic scan (atau on-demand):
- Outdated dependencies (`cargo outdated`, `osv-scanner`)
- Test coverage rendah per modul
- Security vulnerabilities (CVE check)
- Clippy warnings + custom Rust best practices
- Saran proactive refactor / improvement

### 4.15 Adaptive Begawan Style Learning

Cantrik belajar pola coding kamu dari history approval dan interaksi:
- Naming convention yang kamu prefer
- Error handling style
- Architecture pattern yang sering kamu approve vs reject
- Disimpan di **Tier 4 Global Memory** sebagai *"Begawan Preferences"*
- Semakin lama, Cantrik semakin personal dan *"mengerti gurunya"*

### 4.16 LSP Integration (Language Server Protocol)

- Cantrik bisa berjalan sebagai LSP server
- Integrasi real-time suggestion langsung di Neovim, VS Code, Helix, atau editor lain
- CLI tetap sebagai primary interface — LSP adalah layer tambahan, bukan pengganti

### 4.17 Visual Codebase Intelligence

```bash
/visualize callgraph
/visualize architecture
/visualize dependencies
```

Generate Mermaid atau PlantUML diagram langsung di TUI atau export ke file. Berguna untuk onboarding cepat atau code review.

### 4.18 Macro & Recipe System

```bash
cantrik macro record "deploy workflow"
# ... lakukan sequence command ...
cantrik macro stop

# Replay nanti
cantrik macro run "deploy workflow"
```

Reusable recipes untuk workflow yang berulang. Bisa di-share via plugin registry.

### 4.19 `.cantrik/rules.md` — Custom Guardrails

File markdown khusus yang **selalu di-inject ke context** (berbeda dari `skills/` yang contextual):

```markdown
# .cantrik/rules.md
- Selalu gunakan Result<T, E>, jangan unwrap() di production
- Setiap fungsi publik wajib punya rustdoc
- Jangan commit file .env ke repository
- Security: validasi semua user input sebelum proses
- Architecture: ikuti pattern Router → Handler → Service → Repository
```

### 4.20 `cantrik explain` — Code Archaeology Mode

Trace *why* kode ditulis seperti ini via git blame + commit history + PR description:

```bash
cantrik explain src/auth/middleware.rs --why
# → "Fungsi ini direfactor di commit abc123 karena CVE-2024-XXXX.
#    PR #47 menambahkan expiry check setelah incident prod bulan Juli."
```

Sangat berguna untuk onboarding anggota tim baru atau debugging kode lama.

### 4.21 Experiment Mode dengan Auto-Revert

```bash
cantrik experiment "coba ganti HashMap ke BTreeMap di semua cache"
```

Cantrik eksekusi perubahan, jalankan benchmark/test, bandingkan hasilnya, lalu **otomatis revert** jika tidak ada improvement. Tidak perlu manual rollback.

### 4.22 `cantrik review` — Pre-commit AI Review

```bash
cantrik review
# → Analisis diff: security issues, performance, style, test gaps
# → Bisa langsung fix atau skip per issue
```

Bisa dijadikan git pre-commit hook otomatis. Cantrik review kode kamu sebelum commit — bukan setelah.

### 4.23 Context Handoff Protocol

Saat task terlalu panjang atau perlu dilanjutkan developer lain:

```bash
cantrik handoff
# → Menghasilkan .cantrik/handoff-2025-07-01.md
# Berisi: progress, decisions made, next steps, open questions
```

File handoff bisa di-load di sesi berikutnya atau dikirim ke developer lain.

### 4.24 Dependency Intelligence

```bash
cantrik why serde_json
# → "Dipakai di 23 file. Bisa diganti simd-json untuk ~40% speedup."

cantrik upgrade
# → Analisis breaking changes sebelum upgrade, saran per dependency

cantrik audit
# → CVE check + license compatibility report
```

### 4.25 `cantrik teach` — Knowledge Extraction

```bash
cantrik teach
# → Generate ARCHITECTURE.md, ADR (Architecture Decision Records), API docs

cantrik teach --format wiki
# → Export ke format Notion/Obsidian/Confluence-compatible
```

Cantrik bisa menghasilkan dokumentasi dari codebase yang sudah ada — bukan hanya menulis kode baru.

### 4.26 Voice-to-Code & TTS Notification

```bash
cantrik listen
# → Voice input via Whisper lokal (via Ollama)
# → "refactor semua endpoint ke async" → diproses sebagai perintah
```

TTS lokal untuk membacakan respons panjang, notifikasi background, atau error summary — cocok untuk developer yang multitasking. Tersedia sebagai opt-in (config: `voice_enabled = true`).

### 4.27 Session Replay

```bash
cantrik replay sessions/session-2025-07-01.json
```

Rekam seluruh sesi (action + context) ke file. Bisa di-replay untuk debugging apa yang dilakukan Cantrik, atau sharing dengan tim.

---

## 5. Guardrails & Sistem Keamanan

Filosofi Cantrik sebagai asisten setia tercermin dalam sistem keamanannya. Cantrik tidak pernah bertindak melampaui kehendak Begawan.

### Permission Tiers

| Level | Status | Contoh Operasi |
|---|---|---|
| 🔴 | **FORBIDDEN** — hardcoded, tidak bisa di-override | `rm -rf` sistem, akses file di luar project dir, kirim data ke endpoint asing |
| 🟡 | **REQUIRE_APPROVAL** — default on, bisa dikonfigurasi | Write file, eksekusi command, git push/commit, network requests |
| 🟢 | **AUTO_APPROVE** — default | Read file, search codebase, generate suggestion (tanpa apply) |

### Begawan Mode — Autonomy Levels

```toml
# .cantrik/cantrik.toml
[guardrails]
autonomy_level   = "supervised"  # conservative | supervised | autonomous
checkpoint_every = 5             # pause setiap 5 tool call untuk konfirmasi

require_approval = ["delete", "git push", "deploy", "curl", "wget"]
auto_approve     = ["read", "search", "grep", "ls"]
```

### Advanced Sandbox Options

- `bubblewrap` — Linux namespace isolation (default restricted)
- `sandbox-exec` — macOS Seatbelt
- `gVisor` — kernel-level isolation untuk proyek sangat sensitif
- `Docker / Firecracker` — full container isolation

### Audit Log

```
# ~/.local/share/cantrik/audit.log
[2025-07-01 14:23:11] WRITE  src/auth/middleware.rs  model=claude-sonnet-4  cost=$0.003
[2025-07-01 14:23:45] EXEC   cargo test              approved_by=user
[2025-07-01 14:24:01] READ   src/handlers/login.rs   auto_approved
[2025-07-01 14:24:18] DENIED rm -rf ./              reason=forbidden_pattern
```

### Stuck Detection

Cantrik tahu kapan harus berhenti dan meminta bantuan — setelah 3 kali mencoba dengan pendekatan berbeda:

```
  ⚠  Cantrik stuck setelah 3 percobaan.

  Yang sudah dicoba:
  1. Fix import path           → masih error
  2. Update cargo.toml dep     → masih error
  3. Clean build cache         → masih error

  Butuh bantuan Begawan. Error terakhir:
  error[E0308]: mismatched types in `auth/middleware.rs:47`
```

---

## 6. Terminal UX — Ancient Cybernetics CLI

### Prompt & Color Scheme

| Elemen | Warna | Penggunaan |
|---|---|---|
| `ꦕꦤ꧀ꦠꦿꦶꦏ꧀ (cantrik) >` | `#C9A84C` (Gold) | Prompt identifier, highlight penting |
| AI Response text | `#E0E0E0` (Base) | Respons utama dari Cantrik |
| Code blocks | `#C9A84C` + `#C0540A` | Syntax highlighting, Gold & Rust accent |
| Thinking / Logs | `#555566` (Smoke) | System logs, proses berpikir, dimmed |
| Approval prompt | `#DD6B20` (Rust) | Warning, action yang perlu persetujuan |

### Streaming dengan Visual Thinking

```
ꦕꦤ꧀ꦠꦿꦶꦏ꧀ (cantrik) > fix the authentication bug

  ◎ Membaca konteks proyek...              [smoke/dimmed]
  ◎ Searching: "auth" di AST index...
  ◎ Found: src/auth/middleware.rs, src/handlers/login.rs
  ◎ Reading 2 files (847 tokens)...

  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Aku menemukan masalahnya. Di middleware.rs baris 47,    [gold]
  token expiry tidak dicek sebelum decode JWT...
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### Diff Preview Sebelum Apply

```
  Proposed changes to src/auth/middleware.rs:

  - let claims = decode_token(token)?;
  + let claims = decode_token(token)?;
  + if claims.exp < Utc::now().timestamp() {
  +     return Err(AuthError::TokenExpired);
  + }

  SEMANTIC : Menambah expiry validation
  RISK     : Low — backward compatible

  Apply? [Y/n/e(dit)/d(iff)/s(kip)]
```

### Input Modes

| Command | Deskripsi |
|---|---|
| `cantrik` | Interactive REPL mode — percakapan panjang |
| `cantrik "do X"` | One-shot mode — satu perintah, langsung selesai |
| `cantrik plan "X"` | Planning mode — buat rencana dulu, user approve, baru eksekusi |
| `cantrik --watch` | Watch mode — monitor file changes, auto-suggest saat error |
| `cargo build 2>&1 \| cantrik` | Pipe mode — kirim output command langsung ke Cantrik |
| `cantrik --from-clipboard` | Baca dari clipboard — cocok untuk paste error dari browser |
| `cantrik --image ss.png` | Image input — analisis screenshot UI (model vision) |
| `cantrik ask "X"` | Read-only mode — tanya tanpa Cantrik bisa eksekusi apapun |
| `cantrik listen` | Voice input mode — perintah via Whisper lokal |
| `cantrik fix <issue-url>` | SWE-agent mode — fix GitHub issue end-to-end |
| `cantrik background "X"` | Background daemon mode — jalan tanpa terminal aktif |

### Built-in Commands

| Command | Fungsi |
|---|---|
| `/cost` | Tampilkan usage & biaya session ini + bulan ini |
| `/memory` | Tampilkan state memory semua tier |
| `/index` | Re-index codebase secara manual |
| `/plan` | Minta Cantrik buat rencana sebelum eksekusi |
| `/rollback` | Rollback ke checkpoint terakhir |
| `/export` | Export session ke Markdown |
| `/doctor` | Cek kesehatan semua komponen Cantrik |
| `/health` | Tech debt scanner — deps, coverage, CVE, clippy |
| `/visualize` | Generate diagram Mermaid/PlantUML codebase |
| `/macro` | Kelola macro & recipe system |
| `/handoff` | Buat handoff file untuk sesi berikutnya |

### Enhancement UX

- **TUI Split Pane:** thinking log | code preview | semantic diff | approval panel
- **Cultural Wisdom Mode** (opsional, `cultural_wisdom = "light"`): sisipkan peribahasa Jawa relevan sebagai pengingat etika kode atau best practices
  - Contoh: *"Alon-alon waton kelakon"* saat debugging yang kompleks
- **Multi-root Workspace:** dukungan kerja di monorepo atau beberapa project sekaligus
- **Time-aware Context:** Cantrik memberi tahu saat file tidak disentuh lama ("File ini tidak disentuh 8 bulan, mungkin outdated")

---

## 7. Plugin & Skill System

Cantrik menggunakan tiga layer extensibility yang saling melengkapi:

### Layer 1: Skill Files (`.md`) — Context

Context tambahan tentang proyek — di-inject otomatis ke context window berdasarkan relevansi task:

```markdown
# .cantrik/skills/backend.md
## Arsitektur Backend
Proyek ini menggunakan Axum sebagai web framework dengan layer:
- Router → Handler → Service → Repository
- Semua error harus menggunakan tipe AppError dari src/errors.rs
- Database: PostgreSQL via sqlx, async only
```

### Layer 2: Lua Plugins — Logic

Plugin dengan logic — untuk workflow otomatis atau custom tools:

```lua
-- .cantrik/plugins/deploy.lua
function on_task_start(task)
  if task:contains("deploy") then
    cantrik.warn("Ingat: pastikan tests passed sebelum deploy!")
    cantrik.require_approval("deploy")
  end
end

function after_write(file)
  if file:ends_with(".rs") then
    cantrik.suggest("cargo clippy -- -D warnings")
  end
end
```

### Layer 3: WASM Plugins — Advanced

Untuk plugin performa tinggi atau yang ditulis dalam bahasa lain (Go, Python via Wasm):

- Cocok untuk: parser custom, linter khusus, integrator tool berat
- Sandbox penuh — WASM tidak bisa akses filesystem kecuali diberi izin eksplisit
- Language-agnostic: compile apapun ke WASM, jalan di Cantrik

### Plugin Registry (Community)

```bash
# Install plugin dari registry
cantrik skill install git-flow
cantrik skill install docker-helper
cantrik skill install laravel-artisan
cantrik skill install prisma-orm

# Manajemen
cantrik skill list
cantrik skill update
cantrik skill remove git-flow
```

---

## 8. Roadmap Pengembangan

### Phase 0 — Fondasi (Bulan 1–2)

**Tujuan:** Cantrik bisa diinstall dan berfungsi sebagai CLI agent dasar.

1. **Project setup:** Cargo workspace, CI/CD GitHub Actions, linting (clippy), formatting (rustfmt)
2. **CLI scaffold:** clap v4 argument parsing, subcommand structure, shell completion
3. **LLM Bridge v1:** Anthropic + Gemini + Ollama + OpenAI + OpenRouter + Groq, streaming response
4. **Basic REPL:** Interactive mode dengan ratatui, colored output dengan crossterm
5. **Config system:** TOML parsing, global + project config, API key management

### Phase 1 — Core Intelligence (Bulan 3–4)

**Tujuan:** Cantrik bisa membaca dan memahami codebase secara semantik.

6. **tree-sitter integration:** AST parsing untuk Rust, Python, JS/TS, Go, Java, C/C++, PHP, Ruby, SQL, TOML, JSON, YAML, Markdown
7. **LanceDB vector store:** Indexing codebase, semantic search, incremental re-index
8. **Embedding pipeline:** Ollama `nomic-embed-text` sebagai default offline embedder
9. **Session Memory:** SQLite setup, conversation history per-folder, context pruning
10. **File tools:** `read_file`, `write_file` dengan diff preview, approval system

### Phase 2 — Agentic Capabilities (Bulan 5–6)

**Tujuan:** Cantrik bisa menjalankan task multi-step secara otonom dengan guardrails.

11. **Tool system:** `run_command` dengan sandboxing, `git_ops` read-only, `web_fetch` opsional
12. **Checkpointing:** Auto-snapshot sebelum write, rollback command
13. **Audit log:** Setiap action tercatat dengan cost tracking
14. **Stuck detection:** Re-planning logic, failure threshold, human escalation
15. **Multi-agent v1:** Orchestrator + sub-agent spawn, parallel execution via tokio

### Phase 3 — Advanced Features (Bulan 7–9)

**Tujuan:** Cantrik menjadi tools kelas dunia, layak dibandingkan Claude Code.

16. **Background daemon:** systemd/launchd integration, progress persistence, notifikasi
17. **Plugin system:** mlua Lua plugins, wasmtime WASM plugins, plugin registry lokal
18. **Smart routing:** Auto model selection berdasarkan task complexity dan cost budget
19. **MCP integration:** Cantrik sebagai MCP server + consume MCP server lain
20. **Semantic diff:** Risk assessment, affected function analysis, test coverage check
21. **Collaborative mode:** Export/import context, handoff protocol, session sharing
22. **Deep Git-Native Workflow:** Auto-branch, AI commit message, PR automation, `cantrik fix <issue-url>`
23. **Structured Plan & Act Mode:** Dual-agent Planner + Builder + Reviewer sub-agent
24. **Web Research & Browser Tool:** `web_search`, `browse_page`, `fetch_docs` dengan approval
25. **`.cantrik/rules.md`:** Custom guardrails layer, always-injected ke context
26. **LSP Integration:** Cantrik sebagai Language Server untuk Neovim / VS Code / Helix
27. **Visual Codebase Intelligence:** `/visualize` → Mermaid/PlantUML di TUI
28. **Macro & Recipe System:** Record, replay, share workflow
29. **`cantrik review`:** Pre-commit AI review, bisa jadi git hook
30. **`cantrik explain`:** Code archaeology via git blame + history
31. **`cantrik teach`:** Knowledge extraction → ARCHITECTURE.md, ADR, API docs
32. **Experiment Mode:** Auto-revert jika tidak ada improvement
33. **Dependency Intelligence:** `cantrik why`, `cantrik upgrade`, `cantrik audit`
34. **Provenance & Explainability:** Metadata per baris kode yang ditulis Cantrik
35. **Voice-to-Code & TTS:** Whisper lokal input, TTS notifikasi (opt-in)

### Phase 4 — Ecosystem (Bulan 10–12)

**Tujuan:** Membangun komunitas open source yang aktif di seputar Cantrik.

36. **cantrik.dev hub:** Website untuk plugin registry, template sharing, dokumentasi
37. **`cantrik init` templates:** Bootstrap project baru dengan template per framework
38. **Air-gapped mode:** Mode 100% offline untuk enterprise yang tidak boleh kirim kode ke cloud
39. **Package manager integrations:** Homebrew, apt/deb, pacman, Nix flake, winget
40. **VS Code extension:** Side panel yang expose Cantrik capabilities ke editor
41. **Desktop companion app (Tauri):** Monitor daemon + notifikasi di desktop
42. **Tech debt scanner:** `/health` command — production ready
43. **Adaptive Begawan Style Learning:** Belajar dari history approval, personal preferences

### Phase 5 — Maturity & Excellence

**Tujuan:** Cantrik menjadi standar industri untuk open-source CLI agent.

44. **Full autonomous SWE-agent mode:** End-to-end fix GitHub issues dengan high reliability
45. **Agent harness improvements:** Self-reflection loops, better re-planning, visibility dashboard
46. **Self-improvement:** Cantrik bisa menganalisis dan suggest improvement ke codebase Cantrik sendiri
47. **Benchmark formal:** vs SWE-bench, Terminal-Bench
48. **Community-driven recipes & templates** di cantrik.dev
49. **Hybrid cloud execution:** Opt-in via SSH ke instance sendiri untuk task berat

---

## 9. Referensi Konfigurasi

### Global Config (`~/.config/cantrik/config.toml`)

```toml
[ui]
theme         = "ancient-cybernetics"
show_thinking = true     # Tampilkan proses berpikir Cantrik
stream        = true     # Streaming response
language      = "id"     # Bahasa respons Cantrik

[ux]
tui_split_pane    = false          # TUI split pane (Phase 3+)
cultural_wisdom   = "light"        # off | light | full
voice_enabled     = false          # Voice input/output via Whisper
time_aware        = true           # Notif jika file lama tidak disentuh

[memory]
vector_model    = "nomic-embed-text"
index_strategy  = "ast_aware"
compression     = true
anchor_file     = "~/.config/cantrik/anchors.md"

[guardrails]
autonomy_level   = "supervised"    # conservative | supervised | autonomous
checkpoint_every = 5

[routing]
auto_route           = true
max_cost_per_session = 0.50        # USD
max_cost_per_month   = 10.00       # USD

[sandbox]
level = "restricted"               # none | restricted | container
```

### Project Config (`.cantrik/cantrik.toml`)

```toml
[project]
name    = "my-api"
lang    = ["rust", "sql"]
ignore  = ["target/", ".env", "*.log"]

[memory]
vector_model        = "nomic-embed-text"
max_index_size      = "500MB"
reindex_on_git_pull = true

[guardrails]
require_approval = ["delete", "git push", "deploy"]
auto_approve     = ["read", "search"]

[skills]
auto_inject = true
files       = ["backend.md", "database.md", "deploy.md"]

[rules]
custom_rules_file = ".cantrik/rules.md"
adaptive_learning = true

[git]
auto_branch  = true
auto_commit  = false            # false by default — tetap butuh approval
pr_provider  = "github"         # github | gitlab | bitbucket
commit_style = "semantic"

[ux]
voice_enabled   = false
cultural_wisdom = "off"
tui_split_pane  = false
```

### Provider Config (`~/.config/cantrik/providers.toml`)

```toml
[providers.anthropic]
api_key       = "${ANTHROPIC_API_KEY}"
default_model = "claude-sonnet-4"

[providers.gemini]
api_key       = "${GEMINI_API_KEY}"
default_model = "gemini-2.5-flash"

[providers.openai]
api_key       = "${OPENAI_API_KEY}"
default_model = "gpt-4o-mini"

[providers.azure]
api_key            = "${AZURE_OPENAI_API_KEY}"
endpoint           = "https://YOUR_RESOURCE.openai.azure.com"
default_deployment = "gpt-4o"
api_version        = "2024-02-01-preview"

[providers.openrouter]
api_key       = "${OPENROUTER_API_KEY}"
default_model = "anthropic/claude-3.5-sonnet"

[providers.groq]
api_key       = "${GROQ_API_KEY}"
default_model = "llama-3.3-70b-versatile"

[providers.ollama]
base_url      = "http://localhost:11434"
default_model = "llama3.3"
embed_model   = "nomic-embed-text"

[routing]
auto_route     = true
fallback_chain = ["anthropic/claude-sonnet-4", "gemini/gemini-flash", "ollama/llama3.3"]

[routing.thresholds]
simple  = "ollama/llama3.3"
medium  = "anthropic/claude-sonnet-4"
complex = "anthropic/claude-opus-4"
```

---

## 10. Open Source & Kontribusi

### Lisensi

**Cantrik menggunakan MIT License** — bebas digunakan, dimodifikasi, dan didistribusikan, termasuk untuk keperluan komersial, selama attribution dipertahankan.

Cantrik tidak mengambil basis dari Claude Code atau tools lain yang bersifat closed-source. Seluruh codebase dibangun dari nol dengan standar etika open source tertinggi.

### Cara Berkontribusi

1. **Fork & clone** repository
2. **Baca** `CONTRIBUTING.md` dan `CODE_OF_CONDUCT.md`
3. **Pilih issue** dengan label `good-first-issue` atau `help-wanted`
4. **Buat branch** dengan format: `feat/nama-fitur` atau `fix/nama-bug`
5. **Submit PR** dengan deskripsi jelas dan test yang passing

### Standar Kode

- **No unsafe Rust** kecuali ada justifikasi kuat dan di-review ketat
- **Test coverage** minimal 80% untuk semua modul core
- **Dokumentasi** untuk semua public API (rustdoc)
- **clippy** dan rustfmt wajib passing di CI
- **Conventional Commits** untuk semua commit message

### Komunitas

- **GitHub Discussions** — tanya jawab, ide, RFC
- **GitHub Issues** — bug report dan feature request
- **cantrik.dev** — website resmi, dokumentasi, plugin registry (Phase 4)

---

*ꦕꦤ꧀ꦠꦿꦶꦏ꧀ · Cantrik CLI · Open Source · Built with Rust*