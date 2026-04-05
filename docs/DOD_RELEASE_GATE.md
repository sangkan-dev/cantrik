# Release gate — Cantrik vs DEFINITION_OF_DONE.md

Dokumen ini menetapkan **gate rilis** yang direkomendasikan agar keputusan tag/GA tidak membandingkan satu set MUST yang terlalu luas sekaligus.

Rujukan: [DEFINITION_OF_DONE.md](../DEFINITION_OF_DONE.md), matriks bukti: [DOD_VERIFICATION_MATRIX.md](DOD_VERIFICATION_MATRIX.md).

## Gate yang disarankan

| Rilis | MUST wajib PASS | SHOULD / Phase 5 |
|-------|-----------------|------------------|
| **Alpha (CLI preview)** | Phase 0 + kriteria **Global** (keamanan, stabilitas, filosofi) untuk jalur yang di-ship | Non-blocking |
| **Beta** | Alpha + Phase 1 semua MUST | Non-blocking kecuali diputuskan produk |
| **GA / ekosistem** | Beta + Phase 2–3 semua MUST yang relevan dengan fitur yang diiklankan + Phase 4 MUST distribusi/dokumen/kualitas yang dipilih tim | SHOULD Phase 4–5 non-blocking default |

## Praktik operasional

1. Sebelum tag `v*`, jalankan [scripts/dod-auto-smoke.sh](../scripts/dod-auto-smoke.sh) dan pastikan exit 0.
2. Buka [DOD_VERIFICATION_MATRIX.md](DOD_VERIFICATION_MATRIX.md): untuk gate yang dipilih, tidak boleh ada MUST **FAIL**; **PARTIAL** harus punya alasan tertulis dan disetujui maintainer.
3. Baca ringkasan go/no-go: [DOD_GO_NO_GO.md](DOD_GO_NO_GO.md).

## Catatan struktur repo

Workspace saat ini: `cantrik-cli` + `cantrik-core` (modul LLM/RAG/tools di dalam `cantrik-core`). Gate **tidak** mensyaratkan split crate `cantrik-llm` selama substansi MUST terpenuhi — lihat penyelarasan di [DEFINITION_OF_DONE.md](../DEFINITION_OF_DONE.md) (catatan workspace).
