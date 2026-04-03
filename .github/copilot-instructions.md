# Cantrik Copilot Instructions and Rules

## Project Context
- Project: `cantrik` (Open-source AI CLI agent berbasis Rust).
- Sumber requirement utama: `prd/cantrik-doc.js`.
- Tracking delivery: `TASK.md` (model sprint + checklist).
- Prioritas sekarang: membangun fondasi Phase 0 -> Phase 1 dari PRD.

## Core Working Rules
- Selalu mulai dari requirement di PRD dan mapping ke sprint item di `TASK.md`.
- Jangan melakukan perubahan destruktif atau berisiko tanpa persetujuan user.
- Untuk perubahan kode, utamakan patch kecil, terukur, dan mudah direview.
- Jangan menambah dependency baru tanpa alasan teknis yang jelas.
- Jika ada beberapa opsi implementasi, pilih opsi paling sederhana yang memenuhi requirement.

## Rust Engineering Rules
- Gunakan `Result<T, E>` untuk jalur error; hindari `unwrap()` dan `expect()` di production path.
- Gunakan struktur modular saat fitur mulai membesar (pisahkan parser, config, provider, tool).
- Gunakan tipe dan nama yang deskriptif (`snake_case` untuk fungsi/variabel, `PascalCase` untuk type).
- Tambahkan unit test untuk logic non-trivial.
- Pertahankan warning-free build (clippy dan rustfmt friendly).

## Delivery Rules
- Setiap fitur baru harus menyebut acceptance criteria sebelum dianggap selesai.
- Update status task di `TASK.md` saat pekerjaan dimulai (`[/]`) dan saat selesai (`[x]`).
- Jika task melebar dari scope sprint, catat defer reason secara singkat.

## Collaboration Rules
- Jelaskan trade-off teknis secara ringkas dan berbasis dampak.
- Jika blocked oleh asumsi/keputusan produk, minta konfirmasi dengan pertanyaan spesifik.
- Gunakan Bahasa Indonesia untuk penjelasan ke user kecuali diminta lain.
