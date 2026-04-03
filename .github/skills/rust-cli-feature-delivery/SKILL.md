---
name: rust-cli-feature-delivery
description: "Use when implementing or refactoring Cantrik Rust CLI features end-to-end, including command wiring, module design, tests, and TASK.md status updates."
---

# Rust CLI Feature Delivery

## Tujuan
Membantu delivery fitur Rust CLI dari requirement sampai siap merge.

## Kapan Dipakai
- User minta implement subcommand baru.
- User minta refactor modul CLI/config/provider.
- User minta feature dengan acceptance criteria yang jelas.

## Workflow
1. Petakan requirement ke sprint item di `TASK.md`.
2. Desain perubahan minimal dengan boundary modul jelas.
3. Implement kode Rust dengan error handling aman.
4. Tambahkan/ubah test relevan.
5. Jalankan quality gate (`fmt`, `clippy`, `test`) bila tersedia.
6. Update status item sprint terkait di `TASK.md`.

## Checklist Kualitas
- Tidak ada `unwrap()` di jalur runtime.
- Error message actionable.
- Command help text jelas.
- Tidak ada perubahan unrelated.

## Output
- Kode fitur siap review.
- Ringkasan perubahan + risiko + status task terkait.
