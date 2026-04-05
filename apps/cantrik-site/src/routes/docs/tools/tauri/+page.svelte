<svelte:head>
	<title>cantrik-tauri - Dokumentasi Cantrik</title>
	<meta
		name="description"
		content="cantrik-tauri: tray native Tauri yang memantau flag approval Cantrik."
	/>
</svelte:head>

<article
	class="prose prose-sm max-w-none prose-invert prose-headings:font-heading prose-headings:text-gold-bright prose-p:text-ash prose-a:text-gold prose-strong:text-[#e0e0e0] prose-code:text-gold-bright prose-li:text-ash"
>
	<h1>cantrik-tauri</h1>

	<p class="font-mono text-sm text-ash">
		<strong class="text-gold-dim">Satu kalimat:</strong> aplikasi Tauri dengan tray sistem; MVP saat ini
		terbatas, tetap memantau flag approval seperti tray ringan.
	</p>

	<h2>Build</h2>
	<p>Ikuti README aplikasi di repo.</p>
	<pre class="overflow-x-auto rounded bg-andesite-light p-4 font-mono text-xs text-ash"><code
			>cd apps/cantrik-tauri
npm install
npm run build</code
		></pre>
	<p>
		Lalu build Rust di <code>src-tauri</code>, atau jalankan <code>npm run tauri build</code> untuk bundel.
	</p>

	<h2>Flag approval</h2>
	<p>
		Thread latar memantau path default yang sama dengan cantrik-tray. Override path dari config
		proyek belum dibaca oleh app ini; untuk path kustom gunakan cantrik-tray sampai di-wire.
	</p>

	<p class="font-mono text-sm">
		<a
			class="text-gold hover:text-gold-bright"
			href="https://github.com/sangkan-dev/cantrik/blob/main/apps/cantrik-tauri/README.md"
			target="_blank"
			rel="noreferrer">apps/cantrik-tauri/README.md</a
		>
	</p>
</article>
