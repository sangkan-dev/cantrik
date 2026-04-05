<svelte:head>
	<title>Install - Dokumentasi Cantrik</title>
	<meta
		name="description"
		content="Cara install Cantrik dari sumber Rust, biner release, dan catatan PATH."
	/>
</svelte:head>

<article
	class="prose prose-sm max-w-none prose-invert prose-headings:font-heading prose-headings:text-gold-bright prose-p:text-ash prose-a:text-gold prose-strong:text-[#e0e0e0] prose-code:text-gold-bright prose-li:text-ash"
>
	<h1>Install</h1>

	<p class="font-mono text-sm text-ash">
		<strong class="text-gold-dim">TL;DR:</strong> butuh Rust untuk build dari sumber; atau pakai biner
		dari GitHub Releases / paket distro jika tersedia.
	</p>

	<h2>Dari sumber (repo)</h2>
	<pre class="overflow-x-auto rounded bg-andesite-light p-4 font-mono text-xs text-ash"><code
			>git clone https://github.com/sangkan-dev/cantrik.git
cd cantrik
cargo build --release
cargo install --path crates/cantrik-cli</code
		></pre>

	<h2>Biner release</h2>
	<p>
		Lihat
		<a
			class="text-gold hover:text-gold-bright"
			href="https://github.com/sangkan-dev/cantrik/releases"
			target="_blank"
			rel="noreferrer">GitHub Releases</a
		>
		untuk artefak pada tag versi <code>v*</code>.
	</p>

	<h2>PATH</h2>
	<p>
		Pastikan direktori biner <code>cantrik</code> ada di <code>PATH</code>. Ekstensi VS Code
		mengasumsikan
		<code>cantrik</code> bisa dipanggil dari shell.
	</p>

	<h2>Paket komunitas</h2>
	<p class="font-mono text-sm">
		README root mencantumkan Homebrew, .deb, AUR, Nix, Winget - ikuti tautan di repo untuk status
		terbaru.
	</p>
</article>
