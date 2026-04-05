# Go / No-Go rilis — Cantrik (snapshot audit)

Dokumen ini merangkum **keputusan rilis** berdasarkan [DEFINITION_OF_DONE.md](../DEFINITION_OF_DONE.md) dan [DOD_VERIFICATION_MATRIX.md](DOD_VERIFICATION_MATRIX.md).

**Tanggal snapshot:** 2026-04-05  
**Lingkungan verifikasi otomatis:** Linux, Rust stable, `protoc` + well-known protobuf includes tersedia.

---

## Ringkasan eksekutif

| Gate (lihat [DOD_RELEASE_GATE.md](DOD_RELEASE_GATE.md)) | Rekomendasi | Alasan singkat |
|-----------------------------------------------------------|-------------|----------------|
| **Alpha** (Phase 0 + Global) | **Go** dengan catatan | Suite otomatis (fmt, release build, clippy, test, help) hijau; banyak item MANUAL masih perlu konfirmasi di mesin reviewer |
| **Beta** (+ Phase 1 MUST ketat) | **Kondisional** | Butuh penyelesaian matriks MANUAL (indexing, memory, tools end-to-end) — bukti kode kuat untuk path traversal / read_file |
| **GA / Phase 4 MUST penuh** | **No-go saat ini** | DoD mensyaratkan matriks rilis multi-OS di GitHub Releases dan coverage 70% per crate yang belum selaras struktur repo; lihat GAP di matriks |

---

## Blocking (harus ada sebelum menyatakan “memenuhi DoD” untuk gate tersebut)

1. **Phase 4 distribusi multi-platform (jika gate = GA):** workflow rilis saat ini hanya menghasilkan artefak Linux x86_64; expand matrix atau dokumentasikan proses manual dan perbarui DoD jika scope disengaja lebih sempit.
2. **Coverage 70% (jika gate = GA):** crate `cantrik-llm` / `cantrik-rag` / `cantrik-tools` tidak ada sebagai paket terpisah — tentukan metrik pengganti (mis. llvm-cov pada `cantrik-core` + ambang) atau pecah crate.
3. **Item MANUAL yang masih PARTIAL** pada phase target: wajib diuji atau diturunkan ke PARTIAL yang diterima secara eksplisit oleh maintainer (alasan tertulis).

---

## Non-blocking / backlog dokumen

- README status table sebelumnya tidak mencerminkan TASK.md; diselaraskan ke pointer ke TASK + dokumen DoD.
- Template DoD lama `rust-api` diganti penyelarasan ke `rust-cli` / `generic` di [DEFINITION_OF_DONE.md](../DEFINITION_OF_DONE.md).

---

## Tindakan berikutnya (prioritas)

1. Jalankan matriks MANUAL untuk Phase 0–1 di environment CI mirror (Ubuntu + protoc seperti CI).
2. Putuskan apakah **GA** membutuhkan perluasan `.github/workflows/release.yml` atau penyesuaian DoD ke “Linux-only MVP”.
3. Pasang `cargo llvm-cov` (opsional di CI) dan isi angka coverage untuk `cantrik-core`.
