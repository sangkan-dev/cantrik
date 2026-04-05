<svelte:head>
	<title>Session memory - Dokumentasi Cantrik</title>
	<meta name="description" content="Session memory Cantrik: REPL dan konteks percakapan." />
</svelte:head>

<article
	class="prose prose-sm max-w-none prose-invert prose-headings:font-heading prose-headings:text-gold-bright prose-p:text-ash prose-a:text-gold prose-strong:text-[#e0e0e0] prose-code:text-gold-bright prose-li:text-ash"
>
	<h1>Session memory</h1>
	<p class="font-mono text-sm text-ash">
		<strong class="text-gold-dim">Satu kalimat:</strong> dalam REPL, Cantrik menyimpan konteks percakapan
		sesi agar tidak mengulang dari nol tiap baris.
	</p>
	<h2>Kapan aktif</h2>
	<p>
		Jalankan <code>cantrik</code> tanpa subcommand untuk mode interaktif. Perintah seperti
		<code>cantrik ask</code> bersifat sekali jalan dan tidak membawa memori panjang yang sama.
	</p>
	<h2>Di PRD</h2>
	<p>Arsitektur memori bertingkat dan kompresi konteks ada di PRD bagian Memory Architecture.</p>
</article>
