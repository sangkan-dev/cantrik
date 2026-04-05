<script lang="ts">
	import { resolve } from '$app/paths';
	import SoundLink from '$lib/components/SoundLink.svelte';
</script>

<svelte:head>
	<title>Konsep — Dokumentasi Cantrik</title>
	<meta
		name="description"
		content="CLI vs REPL, config global vs proyek, dan apa arti persetujuan (approve) di Cantrik."
	/>
</svelte:head>

<article
	class="prose prose-sm max-w-none prose-invert prose-headings:font-heading prose-headings:text-gold-bright prose-p:text-ash prose-a:text-gold prose-strong:text-[#e0e0e0] prose-code:text-gold-bright prose-li:text-ash"
>
	<h1>Konsep</h1>

	<p class="font-mono text-sm !text-ash">
		<strong class="text-gold-dim">TL;DR:</strong> Cantrik = agen di terminal; kamu tetap “pemilik” keputusan
		untuk aksi yang mengubah sistem.
	</p>

	<h2>CLI vs REPL</h2>
	<ul>
		<li>
			<strong>CLI satu perintah</strong> — contoh: <code>cantrik ask "…"</code> selesai dan keluar.
		</li>
		<li>
			<strong>REPL</strong> — jalankan <code>cantrik</code> tanpa subcommand: sesi interaktif
			berkelanjutan dengan memori sesi (lihat
			<SoundLink href={resolve('/docs/features/session')} class="text-gold"
				>Session memory</SoundLink
			>).
		</li>
	</ul>

	<aside
		class="not-prose my-6 rounded border border-gold/30 bg-andesite-light/40 p-4 font-mono text-sm text-ash"
	>
		<p class="font-heading text-sm font-medium text-gold-dim">Analogi</p>
		<p class="mt-2">
			REPL seperti obrolan panjang dengan teknisi; CLI satu perintah seperti pesan singkat “tolong
			cek ini saja”.
		</p>
	</aside>

	<h2>Config global vs proyek</h2>
	<ul>
		<li>
			<strong>Global</strong> — biasanya di <code>~/.config/cantrik/</code> (API key, default provider).
		</li>
		<li>
			<strong>Proyek</strong> — folder <code>.cantrik/</code> di repo (aturan, harness, konteks lokal).
		</li>
	</ul>
	<p class="font-mono text-sm">
		Detail struktur: lihat bagian konfigurasi di
		<a
			class="text-gold"
			href="https://github.com/sangkan-dev/cantrik/blob/main/prd/cantrik-prd.md"
			target="_blank"
			rel="noreferrer">PRD §9</a
		>.
	</p>

	<h2>Apa itu “approve”?</h2>
	<p>
		Untuk tindakan yang bisa mengubah file atau menjalankan sesuatu di mesinmu, Cantrik dirancang
		agar kamu bisa <strong>menyetujui atau menolak</strong> langkah tersebut — tergantung mode dan guardrail.
	</p>
	<p class="font-mono text-sm">
		Baca ringkasannya di <SoundLink href={resolve('/docs/safety')} class="text-gold"
			>Safety</SoundLink
		> dan PRD §5 (Guardrails).
	</p>
</article>
