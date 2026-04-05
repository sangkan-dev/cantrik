<script lang="ts">
	import { resolve } from '$app/paths';
	import SoundLink from '$lib/components/SoundLink.svelte';
</script>

<svelte:head>
	<title>Start (5 menit) - Dokumentasi Cantrik</title>
	<meta
		name="description"
		content="Install Cantrik, inisialisasi proyek, dan jalankan satu perintah percobaan di terminal."
	/>
</svelte:head>

<article
	class="prose prose-sm max-w-none prose-invert prose-headings:font-heading prose-headings:text-gold-bright prose-p:text-ash prose-a:text-gold prose-strong:text-[#e0e0e0] prose-code:text-gold-bright prose-li:text-ash"
>
	<h1>Start - 5 menit pertama</h1>

	<aside
		class="not-prose my-6 rounded border border-gold/30 bg-andesite-light/40 p-4 font-mono text-sm text-ash"
	>
		<p class="font-heading text-sm font-medium text-gold-dim">Kenapa?</p>
		<p class="mt-2">
			Cantrik seperti asisten di terminal: dia bisa baca kode dan usulkan langkah, tapi untuk hal
			berisiko ia cenderung <strong class="text-[#e0e0e0]">menunggu persetujuanmu</strong> dulu.
		</p>
	</aside>

	<h2>1. Install</h2>
	<p>
		Lihat <SoundLink class="text-gold" href={resolve('/docs/install')}>Install</SoundLink> untuk opsi
		lengkap.
	</p>
	<pre class="overflow-x-auto rounded bg-andesite-light p-4 font-mono text-xs text-ash"><code
			>cargo install --path crates/cantrik-cli</code
		></pre>
	<p>
		Atau unduh biner dari
		<a
			class="text-gold hover:text-gold-bright"
			href="https://github.com/sangkan-dev/cantrik/releases"
			target="_blank"
			rel="noreferrer">GitHub Releases</a
		>.
	</p>

	<h2>2. Bootstrap folder proyek</h2>
	<pre class="overflow-x-auto rounded bg-andesite-light p-4 font-mono text-xs text-ash"><code
			>cantrik init
cantrik init --template rust-cli</code
		></pre>

	<h2>3. Satu perintah percobaan</h2>
	<pre class="overflow-x-auto rounded bg-andesite-light p-4 font-mono text-xs text-ash"><code
			>cantrik ask "jelaskan struktur folder ini secara singkat"</code
		></pre>
	<p>Atau masuk REPL dengan mengetik <code>cantrik</code> saja.</p>

	<h2>Lanjut</h2>
	<ul class="font-mono text-sm">
		<li>
			<SoundLink class="text-gold" href={resolve('/docs/configure')}>Configure provider</SoundLink>
		</li>
		<li><SoundLink class="text-gold" href={resolve('/docs/concepts')}>Konsep</SoundLink></li>
		<li><SoundLink class="text-gold" href={resolve('/docs/safety')}>Safety</SoundLink></li>
	</ul>
</article>
