<script lang="ts">
	import { resolve } from '$app/paths';
</script>

<main class="mx-auto max-w-3xl px-6 py-16">
	<h1 class="font-heading text-3xl font-semibold text-gold-bright">Agent harness (CI artefak)</h1>
	<p class="mt-4 font-mono text-sm text-ash">
		Gunakan keluaran JSON Cantrik untuk dashboard eksternal atau job CI — tanpa backend dinamis di
		site ini.
	</p>

	<h2 class="mt-10 font-heading text-xl font-semibold text-gold-bright">1. Status job lokal</h2>
	<pre
		class="mt-3 overflow-x-auto rounded bg-andesite-lighter p-4 font-mono text-xs text-ash">cantrik status --json</pre>
	<p class="mt-2 font-mono text-sm text-ash">
		Di VS Code, panel webview memanggil perintah ini pada root workspace (lihat extension Cantrik).
	</p>

	<h2 class="mt-10 font-heading text-xl font-semibold text-gold-bright">
		2. Ringkasan berkas untuk CI
	</h2>
	<pre
		class="mt-3 overflow-x-auto rounded bg-andesite-lighter p-4 font-mono text-xs text-ash">cantrik status --write-harness-summary</pre>
	<p class="mt-2 font-mono text-sm text-ash">
		Menulis <code class="text-gold">.cantrik/session-harness-summary.json</code> (daftar job +
		timestamp Unix). Unggah artefak ini dari workflow GitHub Actions dengan
		<code class="text-gold">actions/upload-artifact</code>.
	</p>

	<h2 class="mt-10 font-heading text-xl font-semibold text-gold-bright">3. Plan run summary</h2>
	<p class="mt-2 font-mono text-sm text-ash">
		Alur <code class="text-gold">cantrik plan --run</code> menghasilkan
		<code class="text-gold">.cantrik/plan-run-summary.json</code> — cocok untuk metrik re-plan di pipeline
		terpisah.
	</p>

	<p class="mt-12 font-mono text-sm">
		<a class="text-gold hover:text-gold-bright" href={resolve('/docs')}>← Docs</a>
		·
		<a class="text-gold hover:text-gold-bright" href={resolve('/dashboard')}>Dashboard</a>
	</p>
</main>
