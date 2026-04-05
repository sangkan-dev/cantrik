# Cantrik hub (static site)

Marketing and placeholder hub for **Cantrik**, built from the Sangkan **Ancient Cybernetics** starter (`sangkan-starter` in this repo).

- **Target URL:** `https://cantrik.sangkan.dev` (deploy the `build/` output to your static host / Cloudflare Pages).
- **Stack:** SvelteKit, Svelte 5, Tailwind v4, `@sveltejs/adapter-static`.

```bash
npm ci
npm run check
npm run lint
npm run build
```

Production files are written to `build/`. Plugin list MVP: `static/registry/plugins.json`.

See also [`../README.md`](../README.md) for monorepo layout.

## Backlog signature (opsional)

Rencana peningkatan visual/animasi berat (mis. **GSAP** + ScrambleText, **Three.js / Threlte** + shader CRT) **belum** termasuk dependency saat ini. Jika ditambahkan nanti: lazy-load per rute, ukur ukuran bundle (`npm run build`), pastikan fallback tanpa JS, hormati `prefers-reduced-motion`, dan catat dampak lisensi/build di PR review.
