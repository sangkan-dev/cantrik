<svelte:head>
	<title>VS Code - Dokumentasi Cantrik</title>
	<meta
		name="description"
		content="Ekstensi VS Code Cantrik: doctor, harness, LSP - butuh cantrik di PATH."
	/>
</svelte:head>

<article
	class="prose prose-sm max-w-none prose-invert prose-headings:font-heading prose-headings:text-gold-bright prose-p:text-ash prose-a:text-gold prose-strong:text-[#e0e0e0] prose-code:text-gold-bright prose-li:text-ash"
>
	<h1>Ekstensi VS Code</h1>

	<p class="font-mono text-sm text-ash">
		<strong class="text-gold-dim">TL;DR:</strong> tombol dan perintah palet memanggil cantrik - CLI harus
		ada di PATH.
	</p>

	<h2>Setup dev</h2>
	<pre class="overflow-x-auto rounded bg-andesite-light p-4 font-mono text-xs text-ash"><code
			>cd apps/cantrik-vscode
npm install
npm run compile</code
		></pre>
	<p>Pasang dari VSIX atau buka folder di VS Code untuk pengembangan.</p>

	<h2>Perintah utama</h2>
	<ul>
		<li>Cantrik: Doctor</li>
		<li>Cantrik: Health (audit only)</li>
		<li>Cantrik: Run in workspace folder (doctor)</li>
		<li>Cantrik: Write / Open / Refresh harness summary</li>
		<li>Cantrik: Start LSP / Stop LSP</li>
		<li>Cantrik: Open hub / Open GitHub repository</li>
	</ul>

	<p class="font-mono text-sm">Daftar lengkap di package.json ekstensi di repo.</p>
</article>
