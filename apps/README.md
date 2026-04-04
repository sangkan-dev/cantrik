# Cantrik — aplikasi web & tooling (monorepo)

## Keputusan domain & layout (Sprint 19)

| Keputusan | Pilihan |
|-----------|---------|
| **Host hub** | `https://cantrik.sangkan.dev` (sesuai [TASK.md](../TASK.md); alias `cantrik.dev` bisa diarahkan ke sini jika domain tersedia) |
| **Lokasi kode situs** | Monorepo: [`cantrik-site/`](cantrik-site/) (bukan repo terpisah), supaya versi dokumen dan CLI bisa selaras dalam satu commit |

Situs dibuat dari template **Sangkan Ancient Cybernetics** (asal [`sangkan-starter/`](../sangkan-starter/)).

## `cantrik-site`

SvelteKit + Tailwind v4. Lihat [cantrik-site/README.md](cantrik-site/README.md).

```bash
cd apps/cantrik-site
npm ci
npm run check
npm run build
```

Build statis keluar di `apps/cantrik-site/build/` (adapter-static), siap untuk Cloudflare Pages / hosting statis.
