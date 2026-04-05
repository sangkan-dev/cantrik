# Matriks verifikasi Cantrik vs DEFINITION_OF_DONE.md

Dokumen hidup: isi kolom **Status** saat audit (`PASS` / `PARTIAL` / `FAIL` / `N/A`) dan **Bukti** (perintah, path kode, PR, log).

**Rujukan:** [DEFINITION_OF_DONE.md](../DEFINITION_OF_DONE.md) · **Gate rilis:** [DOD_RELEASE_GATE.md](DOD_RELEASE_GATE.md) · **Ringkasan keputusan:** [DOD_GO_NO_GO.md](DOD_GO_NO_GO.md)

---

## Log verifikasi otomatis terakhir

| Perintah | Hasil | Catatan |
|----------|-------|---------|
| `./scripts/dod-auto-smoke.sh` | PASS | 2026-04-05; memerlukan `protoc` + include well-known types (script mengisi `PROTOC_INCLUDE` jika ditemukan di `/usr/include`, `/usr/local/include`, atau `~/.local/protoc-include`) |
| `cargo doc -p cantrik-core --no-deps` | PASS | Tanpa warning (exit 0) |
| CI `.github/workflows/ci.yml` | Selaras sebagian | Job `rust`: fmt, **check** (bukan `build --release`), clippy, test — untuk DoD “release tanpa warning” gunakan skrip smoke di atas |

---

## Phase 0 — Fondasi

| Item | Tipe | Status | Bukti / catatan |
|------|------|--------|-----------------|
| `cargo build --release` tanpa error/warning | AUTO | PASS | `./scripts/dod-auto-smoke.sh` |
| `cargo test` passing | AUTO | PASS | idem; 90+ tests `cantrik-core`, 32 `cantrik-cli` |
| `cargo clippy -D warnings` | AUTO | PASS | idem; perbaikan `sync_cmd` tests (field_reassign_with_default) |
| `cargo fmt --check` | AUTO | PASS | idem |
| CI hijau (main) | AUTO | MANUAL_REVIEW | Cek Actions di repo; job utama = fmt + check + clippy + test |
| Workspace: cli + core (+ LLM sebagai modul) | MANUAL | PASS | [Cargo.toml](../Cargo.toml): `cantrik-cli`, `cantrik-core`; LLM di `crates/cantrik-core/src/llm/` |
| `cantrik doctor` tanpa panic | AUTO | PASS | `cargo run -p cantrik-cli -- doctor` (jalankan di checkout) |
| Config merge global + project | MANUAL | PARTIAL_REVIEW | `load_merged_config` + `merge` di [config.rs](../crates/cantrik-core/src/config.rs) |
| API key providers.toml + env | MANUAL | PARTIAL_REVIEW | Lihat TASK Sprint 3 + `doctor` redaksi secret |
| Unknown config fields toleran | MANUAL | PARTIAL_REVIEW | Perlu `#[serde(deny_unknown_fields)]` audit per struct — cek deserialisasi TOML |
| `--help` subcommand utama | AUTO | PASS | smoke script: ask, plan, index, doctor |
| `completions bash` | AUTO | PASS | smoke script |
| One-shot / pipe / REPL | MANUAL | PARTIAL_REVIEW | TASK Sprint 2; uji dengan TTY vs pipe |
| LLM streaming / fallback / provider | MANUAL | PARTIAL_REVIEW | TASK Sprint 3 + MVP notes |
| TUI warna / stream / built-in | MANUAL | PARTIAL_REVIEW | TASK Sprint 4 |

---

## Phase 1 — Core Intelligence

| Item | Tipe | Status | Bukti / catatan |
|------|------|--------|-----------------|
| `cantrik index` tanpa crash | AUTO | PASS | Jalankan `cargo run -p cantrik-cli -- index .` di repo kecil |
| .gitignore, biner, ukuran, chunk AST, inkremental, bahasa | MANUAL | PARTIAL_REVIEW | [indexing/](../crates/cantrik-core/src/indexing/), TASK Sprint 5 |
| LanceDB di `.cantrik/index/` | MANUAL | PARTIAL_REVIEW | `.cantrik/index/lance/`; Sprint 6 |
| Session memory SQLite, pruning, anchors, `/memory` | MANUAL | PARTIAL_REVIEW | Sprint 7; pruning heuristik per TASK |
| `read_file` / `write_file` + path project | MANUAL | PASS (kode) | `resolve_path_in_project`, `tool_read_file` di [dispatch.rs](../crates/cantrik-core/src/tool_system/dispatch.rs) |

---

## Phase 2 — Agentic

| Item | Tipe | Status | Bukti / catatan |
|------|------|--------|-----------------|
| Tool registry + tier + sandbox + git_ops | MANUAL | PARTIAL_REVIEW | Sprint 8; macOS sandbox notes TASK |
| Checkpoint / rollback / audit | MANUAL | PARTIAL_REVIEW | Sprint 9; cost stub per TASK |
| Plan / re-plan / stuck | MANUAL | PARTIAL_REVIEW | Sprint 10; act = simulasi MVP |
| Multi-agent | MANUAL | PARTIAL_REVIEW | Sprint 11; Reviewer §4.12 ditunda |

---

## Phase 3 — Advanced

| Item | Tipe | Status | Bukti / catatan |
|------|------|--------|-----------------|
| Background / daemon / status / approval | MANUAL | PARTIAL_REVIEW | Sprint 12 MVP: jeda antar putaran LLM |
| Plugin / skill / Lua / WASM | MANUAL | PARTIAL_REVIEW | Sprint 13 MVP: WASM tanpa WASI, dll. |
| Routing / cost / MCP | MANUAL | PARTIAL_REVIEW | Sprint 14 |
| Semantic diff / handoff / replay / git / web / health / visualize | MANUAL | PARTIAL_REVIEW | Sprint 15–19 |

---

## Phase 4 — Ekosistem

| Item | Tipe | Status | Bukti / catatan |
|------|------|--------|-----------------|
| `--version` vs git tag | AUTO | PARTIAL | Bandingkan [Cargo.toml](../Cargo.toml) `workspace.package.version` dengan tag `v*` |
| Homebrew / apt install bersih | MANUAL | PARTIAL_REVIEW | Formula di `packaging/`; belum diverifikasi di environment bersih otomatis |
| GitHub Releases multi-platform | MANUAL | FAIL vs DoD ketat | [release.yml](../.github/workflows/release.yml): saat ini hanya Linux x86_64 (ubuntu-latest) |
| `cantrik init --template rust-cli` | MANUAL | PASS (kode) | [init_cmd.rs](../crates/cantrik-cli/src/commands/init_cmd.rs) |
| README / CONTRIBUTING | MANUAL | PARTIAL_REVIEW | README perlu selaras progres (lihat pembaruan terbaru) |
| rustdoc public API | AUTO | PASS | `cantrik-core`; ulangi untuk crate lain jika dipisah |
| Dokumentasi publik (site) | MANUAL | PARTIAL_REVIEW | `apps/cantrik-site` → cantrik.sangkan.dev |
| Coverage ≥ 70% (core / llm / rag / tools) | AUTO | N/A / GAP | Crate terpisah tidak ada; ukur modul dengan `cargo llvm-cov` bila terpasang |
| unsafe / unwrap policy | MANUAL | PARTIAL_REVIEW | Audit `rg 'unwrap\\(|expect\\(' crates/` (kecuali test) |
| `/health` offline CVE | MANUAL | PARTIAL_REVIEW | Sprint 19 MVP vs DoD wording |

---

## Phase 5 & SHOULD

Perlakukan sebagai non-blocking default untuk rilis (lihat [DOD_RELEASE_GATE.md](DOD_RELEASE_GATE.md)).

---

## Kriteria global

| Item | Tipe | Status | Bukti / catatan |
|------|------|--------|-----------------|
| API key tidak bocor ke log | AUTO | PARTIAL_REVIEW | Audit `doctor`, audit log writer |
| Path traversal / luar project | AUTO | PASS (kode) | [dispatch.rs](../crates/cantrik-core/src/tool_system/dispatch.rs), [checkpoint/mod.rs](../crates/cantrik-core/src/checkpoint/mod.rs) |
| Forbidden tidak override | MANUAL | PARTIAL_REVIEW | Tool registry + tests |
| Plugin tanpa akses credential | MANUAL | PARTIAL_REVIEW | Sprint 13 WASM/Lua MVP |
| Panic / timeout / SQLite corrupt | MANUAL | PARTIAL_REVIEW | Cari handler di session/index paths |
| Lisensi MIT-compatible | MANUAL | PARTIAL_REVIEW | `cargo deny` / `cargo license` opsional |
| Tulis file hanya via approval | MANUAL | PARTIAL_REVIEW | `tool_write_file` + UI approve |

---

## Cara memperbarui setelah audit

1. Jalankan `./scripts/dod-auto-smoke.sh` dan tempel baris ke tabel **Log verifikasi otomatis**.
2. Perbarui status MANUAL setelah skenario di lingkungan Anda.
3. Sinkronkan temuan FAIL ke [TASK.md](../TASK.md) (lihat § Verifikasi DoD).
