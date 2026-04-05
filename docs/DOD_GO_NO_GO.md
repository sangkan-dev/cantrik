# Go / No-Go rilis — Cantrik (snapshot audit)

Dokumen ini merangkum **keputusan rilis** berdasarkan [DEFINITION_OF_DONE.md](../DEFINITION_OF_DONE.md) dan [DOD_VERIFICATION_MATRIX.md](DOD_VERIFICATION_MATRIX.md).

**Tanggal snapshot:** 2026-04-05  
**Lingkungan verifikasi:** Linux, Rust 1.93.0, protoc tersedia, audit kode via `dod-auto-smoke.sh` + review manual codebase.

---

## Ringkasan eksekutif

| Gate (lihat [DOD_RELEASE_GATE.md](DOD_RELEASE_GATE.md)) | Rekomendasi           | Alasan singkat                                                                                                                                                                                             |
| ------------------------------------------------------- | --------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Alpha** (Phase 0 + Global)                            | ✅ **Go**             | Suite otomatis (fmt, release build, clippy, 122 tests) hijau; security items (path traversal, forbidden tool, API key leak) semua PASS lewat unit test; 5 `unwrap/expect` di production minor dan low-risk |
| **Beta** (+ Phase 1 MUST ketat)                         | ⚠️ **Kondisional**    | Unit test untuk path traversal, read_file, AST chunking, planning, multi-agent, sandbox PASS; butuh verifikasi e2e live (LLM stream, Ollama, SQLite session) di environment nyata                          |
| **GA / Phase 4 MUST penuh**                             | ❌ **No-go saat ini** | Platform binary only Linux x86_64 (DoD requires +aarch64 +macOS); coverage ≥70% belum terukur; README roadmap outdated; dokumentasi publik (cantrik.sangkan.dev) belum live                                |

---

## Blocking (harus ada sebelum menyatakan "memenuhi DoD" gate tersebut)

### Gate Alpha

_Tidak ada blocker yang menghalangi Alpha release._ Semua checklist otomatis hijau dan critical security invariants terbukti via tests.

### Gate Beta (tambahan dari Alpha)

1. **Verifikasi e2e LLM streaming** — butuh uji dengan API key aktif (Anthropic, Gemini) dan Ollama lokal untuk mengkonfirmasi streaming, fallback chain, dan provider switching.
2. **Verifikasi Session SQLite** — lanjutkan sesi REPL setelah tutup terminal, pastikan `anchors.md` dimuat, context pruning aktif.
3. **Uji pipe mode** — `echo "test" | cantrik` di non-TTY environment.
4. **Unknown config field tolerance** — uji deserialisasi TOML dengan field tidak dikenal di beberapa struct krusial.

### Gate GA (tambahan dari Beta)

1. **Multi-platform release binary** — expand `.github/workflows/release.yml` untuk: Linux aarch64, macOS x86_64, macOS aarch64 (atau turunkan scope DoD secara eksplisit ke "Linux MVP").
2. **Coverage ≥ 70%** — instal `cargo llvm-cov` dan ukur coverage `cantrik-core`; jika belum ≥70%, tambahkan tests atau sesuaikan DoD threshold.
3. **README update** — bagian Roadmap masih menunjuk Sprint 1-2; perbarui ke status terkini Sprint 1-19.
4. **Dokumentasi publik live** — deploy `apps/cantrik-site` ke `cantrik.sangkan.dev`.

---

## Temuan positif dari audit

- ✅ **122 tests** semua passing, zero failures
- ✅ **Zero unsafe** di production code (semua `unsafe` blocks ada di `#[cfg(test)]` untuk manipulasi env var di unit tests)
- ✅ **API key tidak bocor** — `doctor.rs` hanya cek `is_ok()`, tidak mencetak nilai key; format audit log tidak include secret params
- ✅ **Path traversal dicegah** — `resolve_path_in_project` + unit test `path_outside_project_rejected_for_read` PASS
- ✅ **Forbidden tools bekerja** — `blocks_rm_rf`, `run_command_blocked_when_forbidden` PASS
- ✅ **cargo doc** bersih tanpa warning
- ✅ **Mutex locks** di `repl.rs` tidak bisa panic secara realistis (thread tidak poisoned)

## Temuan yang perlu perhatian

- ⚠️ **5 `unwrap()`/`expect()` di production path** — semua low-risk tapi tidak sesuai DoD ketat: `repl.rs` (Mutex.lock x3 + guarded hist_cursor), `cultural_wisdom.rs`, `sync_cmd.rs` (current_dir)
- ⚠️ **Release binary hanya Linux x86_64** — ini blocker GA
- ⚠️ **Coverage belum terukur** — `cargo llvm-cov` belum dijalankan

---

## Tindakan berikutnya (prioritas)

1. **[Langsung bisa dikerjakan]** Perbaiki 5 `unwrap()`/`expect()` di production path → ganti dengan `?` atau error handling proper.
2. **[Langsung bisa dikerjakan]** Update README roadmap section agar mencerminkan progres Sprint 1-19.
3. **[Sebelum Beta]** Uji e2e LLM streaming + session memory di environment dengan API key.
4. **[Sebelum GA]** Expand release matrix ke multi-platform atau revisi scope DoD.
5. **[Sebelum GA]** Install `cargo llvm-cov` dan ukur coverage `cantrik-core`.
