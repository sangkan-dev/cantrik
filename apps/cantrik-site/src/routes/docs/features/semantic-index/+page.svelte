<svelte:head>
	<title>Semantic index - Dokumentasi Cantrik</title>
	<meta
		name="description"
		content="Semantic index Cantrik: embeddings lokal, pencarian kode, indexing inkremental."
	/>
</svelte:head>

<article
	class="prose prose-sm max-w-none prose-invert prose-headings:font-heading prose-headings:text-gold-bright prose-p:text-ash prose-a:text-gold prose-strong:text-[#e0e0e0] prose-code:text-gold-bright prose-li:text-ash"
>
	<h1>Semantic index</h1>

	<p class="font-mono text-sm text-ash">
		<strong class="text-gold-dim">Satu kalimat:</strong> Cantrik membaca struktur repo dan menyimpan
		<strong>representasi vektor lokal</strong> supaya pencarian makna (bukan hanya teks persis) lebih
		baik.
	</p>

	<aside
		class="not-prose my-6 rounded border border-gold/30 bg-andesite-light/40 p-4 font-mono text-sm text-ash"
	>
		<p class="font-heading text-sm font-medium text-gold-dim">Analogi</p>
		<p class="mt-2">
			Seperti indeks buku di perpustakaan: tanpa indeks kamu harus baca halaman per halaman.
		</p>
	</aside>

	<h2>Perintah awal</h2>
	<pre class="overflow-x-auto rounded bg-andesite-light p-4 font-mono text-xs text-ash"><code
			>cantrik index ./src</code
		></pre>

	<h2>Di PRD</h2>
	<p>
		Lihat <strong>4.1 Codebase Intelligence (RAG Lokal)</strong> dan bagian embedding / LanceDB di PRD
		untuk detail arsitektur.
	</p>
</article>
