<script lang="ts">
	import { resolve } from '$app/paths';
	import SoundLink from '$lib/components/SoundLink.svelte';
</script>

<svelte:head>
	<title>Configure — Dokumentasi Cantrik</title>
	<meta
		name="description"
		content="File config Cantrik: global config.toml, providers, dan mode offline (Ollama)."
	/>
</svelte:head>

<article
	class="prose prose-sm max-w-none prose-invert prose-headings:font-heading prose-headings:text-gold-bright prose-p:text-ash prose-a:text-gold prose-strong:text-[#e0e0e0] prose-code:text-gold-bright prose-li:text-ash"
>
	<h1>Configure</h1>

	<p class="font-mono text-sm !text-ash">
		<strong class="text-gold-dim">TL;DR:</strong> pengaturan utama di
		<code>~/.config/cantrik/</code>; proyek bisa punya folder <code>.cantrik/</code> sendiri.
	</p>

	<aside
		class="not-prose my-6 rounded border border-gold/30 bg-andesite-light/40 p-4 font-mono text-sm text-ash"
	>
		<p class="font-heading text-sm font-medium text-gold-dim">Kenapa dua tempat?</p>
		<p class="mt-2">
			Global = identitas &amp; kunci API milikmu. Proyek = aturan dan file khusus repo — supaya tim
			bisa commit aturan yang sama.
		</p>
	</aside>

	<h2>File utama</h2>
	<ul>
		<li><code>config.toml</code> — preferensi umum (lihat PRD §9.1).</li>
		<li><code>providers.toml</code> — konfigurasi penyedia LLM (PRD §9.3).</li>
		<li><code>.cantrik/cantrik.toml</code> di repo — override tingkat proyek (PRD §9.2).</li>
	</ul>

	<h2>Multi-provider &amp; offline</h2>
	<p>
		Cantrik mendukung beberapa penyedia (mis. Anthropic, Google Gemini, Ollama). Untuk kerja
		<strong>offline / lokal</strong>, Ollama sering dipakai — detail matriks ada di PRD §3.
	</p>

	<p class="font-mono text-sm">
		Setelah config, coba <SoundLink href={resolve('/docs/start')} class="text-gold">Start</SoundLink
		> atau
		<code>cantrik --debug-config</code> untuk melihat path yang dipakai.
	</p>
</article>
