---
name: sprint-task-sync
description: "Use when user asks to create, update, or realign TASK.md sprint checklists from PRD or recent progress; includes backlog grooming and DoD alignment."
---

# Sprint Task Sync

## Tujuan
Menyelaraskan `TASK.md` dengan PRD, progres aktual, dan scope sprint terbaru.

## Kapan Dipakai
- User minta buat/rapikan task board sprint.
- User minta update status checklist setelah coding.
- User minta re-prioritization sprint atau backlog grooming.

## Input Minimum
- PRD atau ringkasan requirement.
- Kondisi terbaru repo/progres implementasi.

## Langkah Eksekusi
1. Baca requirement relevan di PRD.
2. Cek item di `TASK.md` yang terdampak.
3. Ubah status task (`[ ]` -> `[/]` -> `[x]`) sesuai bukti implementasi.
4. Tambahkan atau geser item jika ada scope baru.
5. Pastikan setiap sprint punya Goal dan DoD yang terukur.

## Output
- `TASK.md` yang ter-update, konsisten, dan siap dipakai tracking harian.
- Catatan singkat perubahan prioritas jika ada.

## Guardrails
- Jangan menandai `[x]` jika belum ada bukti implementasi/validasi.
- Hindari menghapus item lama tanpa alasan.
