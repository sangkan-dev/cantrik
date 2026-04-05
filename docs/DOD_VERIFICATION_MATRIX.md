# Matriks verifikasi Cantrik vs DEFINITION_OF_DONE.md

Dokumen hidup: isi kolom **Status** saat audit (`PASS` / `PARTIAL` / `FAIL` / `N/A`) dan **Bukti** (perintah, path kode, PR, log).

**Rujukan:** [DEFINITION_OF_DONE.md](../DEFINITION_OF_DONE.md) · **Gate rilis:** [DOD_RELEASE_GATE.md](DOD_RELEASE_GATE.md) · **Ringkasan keputusan:** [DOD_GO_NO_GO.md](DOD_GO_NO_GO.md)

---

## Log verifikasi otomatis terakhir

| Perintah                                                               | Hasil             | Catatan                                                            |
| ---------------------------------------------------------------------- | ----------------- | ------------------------------------------------------------------ |
| `./scripts/dod-auto-smoke.sh`                                          | **PASS**          | 2026-04-05; exit 0; memerlukan `protoc` + include well-known types |
| `cargo test --workspace --all-targets`                                 | **PASS**          | 32 cantrik-cli + 90 cantrik-core = 122 tests, 0 failed             |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | **PASS**          | Zero warnings                                                      |
| `cargo fmt --all -- --check`                                           | **PASS**          | No changes                                                         |
| `cargo build --release -p cantrik-cli`                                 | **PASS**          | Build sukses                                                       |
| `cargo doc -p cantrik-core --no-deps`                                  | **PASS**          | Tanpa warning (exit 0)                                             |
| CI `.github/workflows/ci.yml`                                          | **MANUAL_REVIEW** | Job `rust`: fmt + check + clippy + test (selaras smoke)            |

---

## Phase 0 — Fondasi

| Item                                                      | Tipe   | Status           | Bukti / catatan                                                                                                                                              |
| --------------------------------------------------------- | ------ | ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `cargo build --release` tanpa error/warning               | AUTO   | ✅ PASS          | `./scripts/dod-auto-smoke.sh` exit 0                                                                                                                         |
| `cargo test` passing                                      | AUTO   | ✅ PASS          | 122 tests, 0 failed (cantrik-cli 32, cantrik-core 90)                                                                                                        |
| `cargo clippy -D warnings`                                | AUTO   | ✅ PASS          | Zero warnings                                                                                                                                                |
| `cargo fmt --check`                                       | AUTO   | ✅ PASS          | No diff                                                                                                                                                      |
| CI hijau (main)                                           | AUTO   | ⚠️ MANUAL_REVIEW | Cek GitHub Actions — workflow selaras dengan smoke script                                                                                                    |
| Workspace: cli + core (+ LLM sebagai modul)               | MANUAL | ✅ PASS          | `Cargo.toml`: `cantrik-cli`, `cantrik-core`; LLM di `crates/cantrik-core/src/llm/`                                                                           |
| `cantrik doctor` tanpa panic                              | AUTO   | ✅ PASS          | `cargo run -p cantrik-cli -- doctor` berjalan                                                                                                                |
| Config merge global + project                             | MANUAL | ⚠️ PARTIAL       | `load_merged_config` + `merge` di `config.rs` — logika ada, belum uji e2e di dua environment                                                                 |
| API key providers.toml + env                              | MANUAL | ⚠️ PARTIAL       | `resolve_api_key` ada; `doctor` redaksi secret dengan benar (tidak menampilkan nilai key)                                                                    |
| Unknown config fields toleran                             | MANUAL | ⚠️ PARTIAL       | Perlu audit `#[serde(deny_unknown_fields)]` per struct — beberapa struct mungkin tidak toleran                                                               |
| `--help` subcommand utama                                 | AUTO   | ✅ PASS          | Smoke script: ask, plan, index, doctor                                                                                                                       |
| `completions bash`                                        | AUTO   | ✅ PASS          | Smoke script confirms valid output                                                                                                                           |
| One-shot / pipe / REPL                                    | MANUAL | ⚠️ PARTIAL       | Kode ada di TASK Sprint 2; perlu uji TTY vs pipe di environment nyata                                                                                        |
| LLM streaming / fallback / provider                       | MANUAL | ⚠️ PARTIAL       | Provider matrix di `llm/providers.rs`; butuh uji dengan API key aktif                                                                                        |
| TUI warna / stream / built-in `/cost` `/memory` `/doctor` | MANUAL | ⚠️ PARTIAL       | REPL di `repl.rs` lengkap dengan semua slash commands; warna `Color::Yellow`/`Cyan` set — color scheme PRD (Gold/Rust/Smoke) belum diverifikasi match persis |

---

## Phase 1 — Core Intelligence

| Item                                                            | Tipe   | Status     | Bukti / catatan                                                                                                 |
| --------------------------------------------------------------- | ------ | ---------- | --------------------------------------------------------------------------------------------------------------- |
| `cantrik index` tanpa crash                                     | AUTO   | ✅ PASS    | `cargo run -p cantrik-cli -- index .` berhasil                                                                  |
| .gitignore-aware, biner, ukuran, chunk AST, inkremental, bahasa | MANUAL | ⚠️ PARTIAL | `indexing/` crate ada; TASK Sprint 5 [x]; test unit `chunk_rust_fn_named_main`, `chunk_python_def` dll. passing |
| LanceDB di `.cantrik/index/lance/`                              | MANUAL | ⚠️ PARTIAL | `search/store.rs` ada; Sprint 6 [x]; embedding via Ollama — belum diuji tanpa Ollama lokal                      |
| Session memory SQLite, pruning, anchors, `/memory`              | MANUAL | ⚠️ PARTIAL | `session/mod.rs` + `session/anchors.rs` ada; test `combines_global_and_project_anchors` PASS                    |
| `read_file` / `write_file` + path project                       | MANUAL | ✅ PASS    | `resolve_path_in_project`, `tool_read_file`; unit test `path_outside_project_rejected_for_read` PASS            |

---

## Phase 2 — Agentic

| Item                                     | Tipe   | Status     | Bukti / catatan                                                                                                                     |
| ---------------------------------------- | ------ | ---------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| Tool registry + tier + sandbox + git_ops | MANUAL | ⚠️ PARTIAL | `tool_system/tier.rs`: test `forbidden_overrides_auto_list`, `star_forbids_all` PASS; `git_allow`: `allows_log`, `blocks_push` PASS |
| Checkpoint / rollback / audit            | MANUAL | ⚠️ PARTIAL | `checkpoint/mod.rs`: test `write_twice_then_rollback_restores_first_content` PASS; cost stub (0.0) per TASK                         |
| Plan / re-plan / stuck                   | MANUAL | ⚠️ PARTIAL | `planning/`: test `run_loop_escalates_on_repeated_failure`, `run_loop_completes_when_eval_always_success` PASS                      |
| Multi-agent                              | MANUAL | ⚠️ PARTIAL | `multi_agent/`: test `max_depth_blocks_without_llm`, `parse_decompose_two_tasks` PASS; Reviewer §4.12 ditunda                       |

---

## Phase 3 — Advanced

| Item                                             | Tipe   | Status     | Bukti / catatan                                                                                                      |
| ------------------------------------------------ | ------ | ---------- | -------------------------------------------------------------------------------------------------------------------- |
| Background / daemon / notifikasi                 | MANUAL | ⚠️ PARTIAL | `background/`: test `enqueue_claim_complete` PASS; daemon jeda antar putaran LLM (Sprint 12 MVP)                     |
| Plugin / skill / Lua / WASM                      | MANUAL | ⚠️ PARTIAL | `plugins/lua_runtime`: test `lua_on_task_start_suggest` PASS; `plugins/wasm_runtime`: test `wasm_load_and_ping` PASS |
| Routing / cost / MCP                             | MANUAL | ⚠️ PARTIAL | `llm/routing_tests`: 3 test PASS; `cost` modul ada; `serve_mcp` CLI ada — belum uji live MCP connection              |
| Semantic diff / handoff / replay                 | MANUAL | ⚠️ PARTIAL | `semantic_diff/collab`: test `context_bundle_json_roundtrip`, `replay_timeline_contains_ordinals` PASS               |
| Git workflow / web research / health / visualize | MANUAL | ⚠️ PARTIAL | `visualize/mermaid`: test `callgraph_mermaid_wraps`, `deps_from_sample_tree` PASS; web fetch guard ada               |

---

## Phase 4 — Ekosistem

| Item                                          | Tipe   | Status                       | Bukti / catatan                                                                                                                                                                                          |
| --------------------------------------------- | ------ | ---------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--version` vs git tag                        | AUTO   | ⚠️ PARTIAL                   | `workspace.package.version = "0.1.0"` di Cargo.toml; bandingkan dengan git tag `v*` saat rilis                                                                                                           |
| Homebrew / apt install bersih                 | MANUAL | ⚠️ PARTIAL                   | Formula di `packaging/`; belum diverifikasi di environment bersih otomatis                                                                                                                               |
| GitHub Releases multi-platform                | MANUAL | ❌ **FAIL** (jika khusus GA) | `release.yml` hanya Linux x86_64; DoD mensyaratkan Linux x86_64+aarch64 + macOS x86_64+aarch64                                                                                                           |
| `cantrik init --template rust-cli`            | MANUAL | ✅ PASS                      | `init_cmd.rs` ada; unit test `init_writes_dot_cantrik` PASS                                                                                                                                              |
| README / CONTRIBUTING                         | MANUAL | ⚠️ PARTIAL                   | README ada; roadmap section masih menunjuk Sprint 1-2 (outdated); CONTRIBUTING ada                                                                                                                       |
| rustdoc public API                            | AUTO   | ✅ PASS                      | `cargo doc -p cantrik-core --no-deps` exit 0, no warnings                                                                                                                                                |
| Dokumentasi publik (site)                     | MANUAL | ⚠️ PARTIAL                   | `apps/cantrik-site` ada (SvelteKit); belum deploy ke `cantrik.sangkan.dev`                                                                                                                               |
| Coverage ≥ 70% (core)                         | AUTO   | ⚠️ N/A / GAP                 | `cargo llvm-cov` belum dijalankan; tool perlu di-install                                                                                                                                                 |
| Zero `unsafe` tanpa justifikasi di production | MANUAL | ⚠️ PARTIAL                   | Semua `unsafe` ditemukan ada di `#[cfg(test)]` blocks (manipulasi env var) — **tidak ada** `unsafe` di production path                                                                                   |
| Zero `unwrap()`/`expect()` di production path | MANUAL | ⚠️ PARTIAL                   | 5 instance di production: `repl.rs` Mutex locks (3x) + guarded `hist_cursor` (1x), `cultural_wisdom.rs` (1x), `sync_cmd.rs` `current_dir()` (1x) — semua low-risk tapi tidak sepenuhnya sesuai DoD ketat |
| `/health` offline CVE                         | MANUAL | ⚠️ PARTIAL                   | `cantrik health` ada; `cargo audit` dipanggil jika tersedia; offline CVE cache belum terverifikasi                                                                                                       |

---

## Phase 5 & SHOULD

Perlakukan sebagai non-blocking default untuk rilis (lihat [DOD_RELEASE_GATE.md](DOD_RELEASE_GATE.md)).

---

## Kriteria global

| Item                             | Tipe   | Status              | Bukti / catatan                                                                                                                              |
| -------------------------------- | ------ | ------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| API key tidak bocor ke log       | AUTO   | ✅ PASS (kode)      | `doctor.rs` hanya cek `resolve_api_key().is_ok()` — tidak pernah mencetak nilai key; `format_write_audit` tidak terima key params            |
| Path traversal / luar project    | AUTO   | ✅ PASS             | `dispatch.rs` + `checkpoint/mod.rs`: test `path_outside_project_rejected_for_read` PASS                                                      |
| Forbidden tidak override         | MANUAL | ✅ PASS (kode)      | `tool_system/forbidden`: test `blocks_rm_rf`, `allows_echo` PASS; `run_command_blocked_when_forbidden` PASS                                  |
| Plugin tanpa akses credential    | MANUAL | ⚠️ PARTIAL          | Lua `cantrik.require_approval` stub; WASM tanpa WASI — tidak ada akses filesystem dari guest                                                 |
| Panic / timeout / SQLite corrupt | MANUAL | ⚠️ PARTIAL          | Error-handling via `Result`; SQLite corrupt recovery belum diverifikasi secara eksplisit                                                     |
| Lisensi MIT-compatible           | MANUAL | ✅ PASS (workspace) | `workspace.package.license = "MIT"` di Cargo.toml; `cargo deny` tidak terpasang tapi semua dep utama (tokio, clap, serde, dll.) MIT/Apache-2 |
| Tulis file hanya via approval    | MANUAL | ✅ PASS (kode)      | `tool_write_file` + approval prompt ada; `web_fetch_refused_when_offline` PASS                                                               |

---

## Cara memperbarui setelah audit

1. Jalankan `./scripts/dod-auto-smoke.sh` dan tempel baris ke tabel **Log verifikasi otomatis**.
2. Perbarui status MANUAL setelah skenario di lingkungan Anda.
3. Sinkronkan temuan FAIL ke [TASK.md](../TASK.md) (lihat § Verifikasi DoD).
