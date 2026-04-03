---
name: sprint-task-sync
description: Menyelaraskan TASK.md dengan PRD, progres, dan scope sprint; membuat atau merapikan checklist sprint, memperbarui status, backlog grooming, dan menyelaraskan DoD. Digunakan ketika pengguna meminta task board sprint, update checklist setelah coding, re-prioritisasi sprint, atau grooming backlog.
---

# Sprint task sync

## Tujuan
Menyelaraskan `TASK.md` dengan PRD, progres aktual, dan scope sprint terbaru.

## Kapan dipakai
- Permintaan membuat atau merapikan task board sprint.
- Permintaan update status checklist setelah coding.
- Permintaan re-prioritisasi sprint atau backlog grooming.

## Input minimum
- PRD atau ringkasan requirement.
- Kondisi terbaru repo atau progres implementasi.

## Langkah eksekusi
1. Baca requirement relevan di PRD.
2. Cek item di `TASK.md` yang terdampak.
3. Ubah status task (`[ ]` → `[/]` → `[x]`) sesuai bukti implementasi.
4. Tambahkan atau geser item jika ada scope baru.
5. Pastikan setiap sprint punya goal dan DoD yang terukur.

## Output
- `TASK.md` ter-update, konsisten, dan siap dipakai tracking harian.
- Catatan singkat perubahan prioritas jika ada.

## Guardrails
- Jangan tandai `[x]` tanpa bukti implementasi atau validasi.
- Hindari menghapus item lama tanpa alasan.
