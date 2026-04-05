# DEFINITION_OF_DONE.md — Cantrik

Dokumen ini adalah **checklist objektif dan verifiable** untuk menentukan apakah Cantrik
dianggap selesai di setiap phase. Dirancang agar bisa diverifikasi oleh AI reviewer
manapun hanya dengan membaca kode dan menjalankan perintah.

Reviewer dapat menggunakan dokumen ini bersama `prd/cantrik-prd.md` dan `TASK.md`
sebagai referensi utama.

**Gate rilis & matriks audit:** lihat `docs/DOD_RELEASE_GATE.md`, `docs/DOD_VERIFICATION_MATRIX.md`,
`docs/DOD_GO_NO_GO.md`, dan skrip `./scripts/dod-auto-smoke.sh`.

---

## Cara Menggunakan Dokumen Ini

1. Setiap item memiliki **kriteria verifikasi** — cara konkret untuk mengecek apakah item terpenuhi
2. Item bertanda `[AUTO]` bisa dicek otomatis (CI, test, command)
3. Item bertanda `[MANUAL]` butuh review kode atau uji coba manual
4. Sebuah Phase dianggap selesai jika **semua item wajib** (`MUST`) terpenuhi
5. Item bertanda `SHOULD` adalah target kualitas — boleh ada 1-2 yang pending tapi harus ada alasan jelas

---

## Phase 0 — Fondasi

### Engineering & Tooling `MUST`
- `[AUTO]` `cargo build --release` berhasil tanpa error dan tanpa warning
- `[AUTO]` `cargo test` semua test passing
- `[AUTO]` `cargo clippy -- -D warnings` zero warning
- `[AUTO]` `cargo fmt --check` tidak ada perubahan formatting
- `[AUTO]` CI GitHub Actions hijau untuk semua job (build, test, clippy, fmt) di push ke `main`
- `[MANUAL]` Workspace multi-crate terdefinisi di root `Cargo.toml` dengan crate minimal: `cantrik-cli`, `cantrik-core`, serta **substansi LLM** (boleh berupa crate terpisah `cantrik-llm` atau modul di `cantrik-core`, mis. `crates/cantrik-core/src/llm/`)

### Config System `MUST`
- `[AUTO]` `cantrik doctor` berjalan tanpa panic
- `[MANUAL]` `~/.config/cantrik/config.toml` dan `.cantrik/cantrik.toml` di-load dengan benar; project config override global config
- `[MANUAL]` API key bisa dibaca dari `providers.toml` maupun environment variable (`ANTHROPIC_API_KEY`, `GEMINI_API_KEY`, dst.)
- `[MANUAL]` Config dengan field tidak dikenal tidak menyebabkan crash (toleran terhadap unknown fields)

### CLI & UX `MUST`
- `[AUTO]` `cantrik --help` menampilkan semua subcommand
- `[AUTO]` `cantrik ask --help`, `cantrik plan --help`, `cantrik index --help`, `cantrik doctor --help` menampilkan usage yang jelas
- `[AUTO]` `cantrik completions bash` menghasilkan output shell completion yang valid
- `[MANUAL]` One-shot mode: `cantrik "pertanyaan"` langsung mendapat respons streaming
- `[MANUAL]` Pipe mode: `echo "fix this" | cantrik` bekerja saat stdin bukan TTY
- `[MANUAL]` REPL mode: `cantrik` tanpa argumen masuk ke interactive loop; `exit`/`quit`/EOF keluar dengan bersih

### LLM Bridge `MUST`
- `[MANUAL]` Streaming response bekerja untuk minimal: Anthropic Claude, Google Gemini, Ollama
- `[MANUAL]` Fallback chain bekerja: jika provider pertama gagal (rate limit / tidak tersedia), otomatis coba provider berikutnya
- `[MANUAL]` Error provider ditampilkan dengan pesan yang jelas, bukan panic
- `[MANUAL]` Provider OpenAI, Azure, OpenRouter, Groq terdaftar dan bisa diaktifkan via config

### TUI `MUST`
- `[MANUAL]` Thinking log ditampilkan dengan warna smoke/dimmed, berbeda dari respons utama
- `[MANUAL]` Respons AI di-stream karakter per karakter, bukan muncul sekaligus
- `[MANUAL]` `/cost`, `/memory`, `/doctor` bisa dipanggil di dalam REPL
- `[MANUAL]` Color scheme sesuai PRD: Gold `#C9A84C`, Rust `#C0540A`, Smoke `#555566`

---

## Phase 1 — Core Intelligence

### AST & Codebase Indexing `MUST`
- `[AUTO]` `cantrik index` berhasil memproses folder project tanpa crash
- `[MANUAL]` File di `.gitignore` tidak ikut di-index
- `[MANUAL]` File biner dan file > threshold ukuran (default: 1MB) dilewati
- `[MANUAL]` Chunk yang dihasilkan berbasis batas fungsi/class, bukan panjang karakter naif
- `[MANUAL]` Re-index inkremental: hanya file yang berubah sejak index terakhir yang diproses ulang
- `[MANUAL]` Bahasa yang didukung minimal: Rust, Python, JavaScript, TypeScript, Go

### Vector Store `MUST`
- `[MANUAL]` Index tersimpan di `.cantrik/index/` dalam format LanceDB
- `[MANUAL]` Query semantik mengembalikan hasil yang relevan (bukan random)
- `[MANUAL]` Embedding default berjalan 100% lokal via Ollama `nomic-embed-text` — tidak ada request ke cloud saat indexing
- `[MANUAL]` `cantrik index` selesai dalam waktu wajar untuk proyek ~10.000 baris kode (< 60 detik)

### Session Memory `MUST`
- `[MANUAL]` History percakapan tersimpan di SQLite dan bisa dilanjutkan saat REPL dibuka ulang di folder yang sama
- `[MANUAL]` Context pruning aktif: saat mendekati batas token, pesan lama diringkas, bukan di-drop mentah
- `[MANUAL]` `anchors.md` di-load dan selalu ada di context setiap sesi
- `[MANUAL]` `/memory` menampilkan state tier 1–4 yang akurat

### File Tools `MUST`
- `[MANUAL]` `read_file` membaca file dan mengembalikan konten; path di luar project directory ditolak
- `[MANUAL]` `write_file` selalu menampilkan diff preview sebelum menulis
- `[MANUAL]` `write_file` meminta approval user (`Y/n`) sebelum apply; default `n` jika tidak ada input
- `[MANUAL]` Tidak ada file yang tertulis tanpa melalui jalur approval

---

## Phase 2 — Agentic Capabilities

### Tool System & Sandbox `MUST`
- `[MANUAL]` Semua tool terdaftar di registry dengan tier permission yang benar (auto/require/forbidden)
- `[MANUAL]` `run_command` tidak bisa dieksekusi tanpa approval user
- `[MANUAL]` Command yang masuk daftar `forbidden` (contoh: `rm -rf /`) ditolak dengan pesan jelas, bahkan jika user coba approve
- `[MANUAL]` Sandbox `restricted` aktif secara default untuk `run_command`
- `[MANUAL]` `git_ops` read-only: `git status`, `git diff`, `git log` bisa berjalan tanpa approval; `git push` butuh approval

### Checkpoint & Rollback `MUST`
- `[MANUAL]` Checkpoint dibuat otomatis sebelum setiap operasi `write_file`
- `[MANUAL]` Checkpoint tersimpan di `.cantrik/checkpoints/` dengan format `checkpoint-NNN-<slug>/`
- `[AUTO]` `cantrik rollback --list` menampilkan daftar checkpoint yang ada
- `[MANUAL]` `cantrik rollback` mengembalikan file ke kondisi sebelum write; file yang di-restore identik byte-per-byte

### Audit Log `MUST`
- `[MANUAL]` Setiap action (READ, WRITE, EXEC, DENIED) tercatat di `~/.local/share/cantrik/audit.log`
- `[MANUAL]` Format log: `[timestamp] ACTION path/command model=X cost=$Y`
- `[MANUAL]` Log bersifat append-only — tidak ada mekanisme auto-delete
- `[MANUAL]` DENIED tercatat untuk setiap aksi yang ditolak (forbidden pattern atau user reject approval)

### Planning & Re-planning `MUST`
- `[MANUAL]` `cantrik plan "task"` menghasilkan rencana step-by-step sebelum eksekusi
- `[MANUAL]` Setelah setiap step, Cantrik mengevaluasi hasil; jika gagal, re-plan dengan informasi baru
- `[MANUAL]` Setelah 3 kali gagal dengan pendekatan berbeda, Cantrik berhenti dan menampilkan ringkasan percobaan
- `[MANUAL]` Pesan stuck menampilkan: apa yang dicoba, error terakhir, dan saran langkah manual

### Multi-Agent `MUST`
- `[MANUAL]` Orchestrator bisa spawn minimal 2 sub-agent yang berjalan paralel
- `[MANUAL]` Setiap sub-agent punya context window yang terisolasi — tidak ada shared mutable state
- `[MANUAL]` Jika satu sub-agent gagal/panic, sub-agent lain tetap berjalan
- `[MANUAL]` Depth limit sub-agent aktif (default: 3); spawn di luar batas ditolak dengan error jelas
- `[MANUAL]` Summary dari sub-agent (bukan full context) yang dikirim ke orchestrator

---

## Phase 3 — Advanced Features

### Background Daemon `MUST`
- `[MANUAL]` `cantrik background "task"` menjalankan proses yang tetap hidup setelah terminal ditutup
- `[MANUAL]` `cantrik status` menampilkan progress task yang sedang berjalan di background
- `[MANUAL]` Jika task background membutuhkan approval, proses di-pause dan notifikasi terkirim
- `[MANUAL]` Progress tersimpan ke SQLite — task bisa dilanjutkan jika daemon restart

### Plugin System `MUST`
- `[MANUAL]` `.cantrik/skills/*.md` di-inject otomatis ke context; file yang tidak relevan tidak ikut di-inject
- `[MANUAL]` `.cantrik/rules.md` selalu di-inject ke setiap sesi, tidak peduli task apapun
- `[MANUAL]` Plugin Lua bisa dipanggil via hook `on_task_start` dan `after_write`
- `[MANUAL]` Plugin Lua yang crash tidak menyebabkan Cantrik ikut crash (isolated error handling)
- `[MANUAL]` Plugin WASM tidak bisa akses filesystem kecuali diberi izin eksplisit lewat config
- `[AUTO]` `cantrik skill list` menampilkan semua plugin yang terinstall

### Smart Routing `MUST`
- `[MANUAL]` Task yang terklasifikasi "simple" menggunakan model di `routing.thresholds.simple`
- `[MANUAL]` Task yang terklasifikasi "complex" menggunakan model di `routing.thresholds.complex`
- `[MANUAL]` Jika `max_cost_per_session` tercapai, Cantrik memperingatkan user sebelum lanjut
- `[AUTO]` `/cost` menampilkan biaya session aktif dan total bulan ini dengan angka yang akurat

### MCP Integration `MUST`
- `[MANUAL]` `cantrik serve --mcp` menjalankan MCP server yang bisa dikoneksi dari Claude Desktop atau Cursor
- `[MANUAL]` Tool dasar Cantrik (read_file, search, run_command) tersedia sebagai MCP tools
- `[MANUAL]` Cantrik bisa memanggil tool dari MCP server eksternal yang dikonfigurasi

### Semantic Diff `MUST`
- `[MANUAL]` Sebelum apply write, ditampilkan: daftar fungsi/file terdampak, level risk (Low/Medium/High), dan apakah ada test coverage untuk perubahan tersebut
- `[MANUAL]` Risk `High` secara default meminta konfirmasi eksplisit meski user sudah set `auto_approve`

### Git-Native Workflow `MUST`
- `[MANUAL]` `cantrik commit "deskripsi"` menghasilkan commit message AI-generated + meminta approval sebelum commit
- `[MANUAL]` Branch baru dibuat otomatis per task saat `auto_branch = true`
- `[MANUAL]` `cantrik pr create "judul"` membuat Pull Request ke GitHub atau GitLab
- `[MANUAL]` Conflict Git terdeteksi dan ditampilkan dengan saran resolusi

### Plan & Act Mode `MUST`
- `[MANUAL]` Planner sub-agent tidak bisa memanggil tool yang memodifikasi file atau menjalankan command
- `[MANUAL]` Builder sub-agent hanya aktif setelah user approve rencana dari Planner
- `[MANUAL]` Reviewer sub-agent menjalankan lint/test setelah setiap langkah Builder dan melaporkan hasilnya

### Web Research `MUST`
- `[MANUAL]` `web_search`, `browse_page`, `fetch_docs` hanya bisa dipanggil setelah approval user
- `[MANUAL]` Data yang di-fetch tidak disimpan ke luar project directory tanpa izin eksplisit

### Tools Tambahan `MUST`
- `[MANUAL]` `cantrik review` menghasilkan analisis diff: security, performance, style, test gaps
- `[MANUAL]` `cantrik explain <file> --why` menampilkan riwayat perubahan file via git blame + commit message
- `[MANUAL]` `cantrik audit` menampilkan CVE yang ditemukan di dependencies dan license compatibility
- `[MANUAL]` `/health` menampilkan laporan: outdated deps, test coverage, clippy warnings, security issues
- `[MANUAL]` `cantrik handoff` menghasilkan file `.cantrik/handoff-YYYY-MM-DD.md` yang berisi progress, keputusan, dan next steps
- `[MANUAL]` `/visualize callgraph` menghasilkan output Mermaid yang valid

### Provenance `SHOULD`
- `[MANUAL]` Kode yang ditulis Cantrik memiliki metadata: timestamp, model, task, confidence level

### Voice `SHOULD`
- `[MANUAL]` `cantrik listen` menerima input suara dan memprosesnya sebagai perintah teks

---

## Phase 4 — Ecosystem

### Distribusi `MUST`
- `[AUTO]` `cantrik --version` menampilkan versi yang konsisten dengan `workspace.package.version` di root `Cargo.toml` dan, saat rilis dari tag, dengan tag git `v*` (verifikasi manual: bandingkan output CLI dengan tag)
- `[MANUAL]` Instalasi via minimal satu package manager berhasil di environment bersih: Homebrew (macOS) atau apt (Ubuntu)
- `[MANUAL]` Binary release tersedia di GitHub Releases untuk: Linux x86_64, Linux aarch64, macOS x86_64, macOS aarch64
- `[MANUAL]` `cantrik init --template rust-cli` (atau `generic`) menghasilkan struktur project yang valid dan siap dipakai

### Dokumentasi `MUST`
- `[MANUAL]` `README.md` berisi: deskripsi singkat, cara install, quickstart 5 menit, link ke docs lengkap
- `[MANUAL]` `CONTRIBUTING.md` berisi: cara setup dev environment, cara run test, cara submit PR
- `[MANUAL]` Semua public API crate memiliki rustdoc (`cargo doc --no-deps` berhasil tanpa warning)
- `[MANUAL]` Situs dokumentasi publik tersedia (mis. `cantrik.sangkan.dev` dari `apps/cantrik-site`, atau GitHub Pages) dengan dokumentasi minimal yang bisa diakses publik

### Kualitas Kode `MUST`
- `[AUTO]` Test coverage keseluruhan ≥ 70% untuk substansi yang setara: minimal crate `cantrik-core`, serta modul LLM / RAG / tools di dalamnya (jika belum dipecah menjadi crate `cantrik-llm`, `cantrik-rag`, `cantrik-tools` — ukur per paket yang ada, mis. `cargo llvm-cov -p cantrik-core`)
- `[AUTO]` Zero `unsafe` block tanpa komentar justifikasi yang menjelaskan kenapa diperlukan
- `[AUTO]` Zero `unwrap()` atau `expect()` di path production (boleh di test)
- `[MANUAL]` Semua error menggunakan tipe error yang proper (`thiserror`), bukan `Box<dyn Error>` di API publik

### Tech Debt Scanner `MUST`
- `[AUTO]` `/health` bisa berjalan tanpa internet (untuk CVE check lokal dari cached database)
- `[MANUAL]` Output `/health` mencakup: versi dependency vs latest, jumlah CVE ditemukan, test coverage per modul, jumlah clippy warning

### Adaptive Learning `SHOULD`
- `[MANUAL]` Setelah 10+ interaksi, Cantrik menunjukkan preferensi yang konsisten dengan pola approval user (naming convention, error handling style, dll.)

---

## Phase 5 — Maturity (Bonus)

### SWE-Agent Mode `SHOULD`
- `[MANUAL]` `cantrik fix <github-issue-url>` menghasilkan PR yang bisa di-review manusia tanpa perlu edit manual untuk kasus sederhana (bug fix dengan test)
- `[MANUAL]` Success rate minimal 30% pada subset SWE-bench yang relevan (Rust/Python issues)

### Self-improvement `SHOULD`
- `[MANUAL]` `cantrik` bisa menganalisis codebase Cantrik sendiri dan menghasilkan saran improvement yang valid
- `[MANUAL]` Saran dari Cantrik untuk Cantrik sudah pernah menghasilkan minimal satu PR yang di-merge

---

## Kriteria Global — Berlaku di Semua Phase

Ini harus terpenuhi sejak Phase 0 dan dipertahankan hingga Phase 5:

### Keamanan `MUST`
- `[AUTO]` Tidak ada API key yang ter-log di audit log, stdout, atau stderr dalam kondisi apapun
- `[AUTO]` Tidak ada path traversal yang memungkinkan akses file di luar project directory
- `[MANUAL]` Operasi `forbidden` tidak bisa di-override oleh config, plugin, atau user input apapun
- `[MANUAL]` Plugin Lua/WASM tidak bisa mengakses API key atau credential user

### Stabilitas `MUST`
- `[MANUAL]` Cantrik tidak panic pada input yang tidak terduga — semua error di-handle dengan pesan yang jelas
- `[MANUAL]` Cantrik tidak hang / infinite loop — semua operasi punya timeout yang dikonfigurasi
- `[MANUAL]` Jika SQLite corrupt atau index rusak, Cantrik menawarkan untuk reset, bukan crash

### Filosofi `MUST`
- `[MANUAL]` Tidak ada satu pun baris kode yang berasal dari Claude Code leaked source atau codebase closed-source lainnya
- `[MANUAL]` Semua dependensi memiliki lisensi yang kompatibel dengan MIT (tidak ada GPL atau AGPL di dependency tree runtime)
- `[MANUAL]` Semua kode yang ditulis Cantrik ke filesystem user memerlukan approval — tidak ada pengecualian

---

## Template Review Prompt untuk AI Reviewer

Jika ingin minta AI lain untuk review, gunakan prompt berikut:

```
Kamu adalah reviewer untuk project Cantrik — open-source CLI AI agent yang dibangun dengan Rust.

Baca dokumen berikut sebagai referensi:
1. prd/cantrik-prd.md — Product Requirements Document lengkap
2. DEFINITION_OF_DONE.md — Checklist ini (yang sedang kamu baca)
3. TASK.md — Sprint board dan status implementasi

Kemudian baca kode di repository ini dan laporkan:

A. PHASE SAAT INI
   - Phase mana yang sedang dikerjakan berdasarkan TASK.md?
   - Sprint mana yang on-progress?

B. CHECKLIST STATUS
   - Untuk setiap item [MUST] di phase yang sudah diklaim selesai:
     berikan status: ✅ PASS / ❌ FAIL / ⚠️ PARTIAL
   - Sertakan bukti: nama file, baris kode, atau hasil command

C. TEMUAN KRITIS
   - Item MUST yang FAIL — wajib diperbaiki sebelum lanjut
   - Potensi security issue (path traversal, API key leak, dll.)
   - Pelanggaran filosofi (unsafe tanpa justifikasi, unwrap di production, dll.)

D. REKOMENDASI
   - 3 hal paling penting untuk dikerjakan selanjutnya
   - Estimasi apakah sprint berikutnya realistis berdasarkan kode saat ini

Format output: gunakan header yang jelas, checklist dengan emoji status,
dan sertakan snippet kode yang relevan sebagai bukti.
```

---

*Dokumen ini adalah living document — update setiap kali ada perubahan scope di PRD.*
*Last updated: 2026-04-05 — penyelarasan workspace, template init, coverage, dan dokumen publik dengan struktur repo saat ini.*