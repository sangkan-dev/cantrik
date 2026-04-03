---
description: "Use when updating PRD-derived planning docs, sprint breakdowns, and TASK.md checklist tracking."
applyTo: "TASK.md,prd/**"
---

# Planning and Task Tracking Rules

## Source of Truth
- PRD adalah referensi utama arah produk.
- `TASK.md` adalah referensi utama status eksekusi.

## Sprint Update Convention
- Gunakan status: `[ ]`, `[/]`, `[x]`.
- Saat pekerjaan dimulai: ubah item jadi `[/]`.
- Saat selesai dan tervalidasi: ubah item jadi `[x]`.

## Scope Management
- Jika requirement baru muncul, masukkan ke sprint aktif hanya jika effort kecil.
- Jika effort besar, pindahkan ke sprint berikutnya atau backlog dengan alasan singkat.

## Definition of Done Hygiene
- Jangan tandai selesai bila acceptance criteria belum terpenuhi.
- Cantumkan outcome yang bisa diverifikasi (contoh: command berhasil jalan, test passing).

## Documentation Quality
- Tulis task singkat, action-oriented, dan measurable.
- Hindari task abstrak seperti "improve system" tanpa indikator selesai.
