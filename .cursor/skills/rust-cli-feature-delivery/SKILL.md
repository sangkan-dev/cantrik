---
name: rust-cli-feature-delivery
description: Mengimplementasikan atau merefaktor fitur Rust CLI Cantrik end-to-end (wiring perintah, desain modul, tes, pembaruan TASK.md). Digunakan ketika pengguna meminta subcommand baru, refactor modul CLI/config/provider, atau fitur dengan acceptance criteria yang jelas.
---

# Rust CLI feature delivery

## Tujuan
Membantu delivery fitur Rust CLI dari requirement sampai siap merge.

## Kapan dipakai
- Implementasi subcommand baru.
- Refactor modul CLI, config, atau provider.
- Fitur dengan acceptance criteria yang jelas.

## Workflow
1. Petakan requirement ke item sprint di `TASK.md`.
2. Desain perubahan minimal dengan batas modul jelas.
3. Implementasi Rust dengan penanganan error yang aman.
4. Tambahkan atau ubah test yang relevan.
5. Jalankan quality gate (`fmt`, `clippy`, `test`) bila memungkinkan.
6. Update status item sprint terkait di `TASK.md`.

## Checklist kualitas
- Tanpa `unwrap()` di jalur runtime.
- Pesan error actionable.
- Help text perintah jelas.
- Tanpa perubahan tidak terkait.

## Output
- Kode fitur siap review.
- Ringkasan perubahan, risiko, dan status task terkait.
