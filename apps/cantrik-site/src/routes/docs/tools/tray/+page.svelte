<script lang="ts">
	import { resolve } from '$app/paths';
</script>

<svelte:head>
	<title>cantrik-tray - Dokumentasi Cantrik</title>
	<meta
		name="description"
		content="cantrik-tray: notifikasi desktop saat file flag approval Cantrik muncul."
	/>
</svelte:head>

<article
	class="prose prose-sm max-w-none prose-invert prose-headings:font-heading prose-headings:text-gold-bright prose-p:text-ash prose-a:text-gold prose-strong:text-[#e0e0e0] prose-code:text-gold-bright prose-li:text-ash"
>
	<h1>cantrik-tray</h1>

	<p class="font-mono text-sm text-ash">
		<strong class="text-gold-dim">Satu kalimat:</strong> program kecil yang
		<strong>memantau</strong>
		file flag dan memberi <strong>notifikasi desktop</strong> ketika CLI meminta perhatianmu.
	</p>

	<aside
		class="not-prose my-6 rounded border border-gold/30 bg-andesite-light/40 p-4 font-mono text-sm text-ash"
	>
		<p class="font-heading text-sm font-medium text-gold-dim">Analogi</p>
		<p class="mt-2">
			Seperti lonceng pintu: CLI menarik tali (menulis flag), tray membunyikan lonceng.
		</p>
	</aside>

	<h2>Jalankan</h2>
	<p class="font-mono text-sm">Dari folder <code>apps/cantrik-tray</code> di repo:</p>
	<pre class="overflow-x-auto rounded bg-andesite-light p-4 font-mono text-xs text-ash"><code
			>cargo run</code
		></pre>

	<h2>Path flag</h2>
	<p>
		Default mengikuti data directory OS, misalnya <code
			>~/.local/share/cantrik/approval-pending.flag</code
		>
		(pada Linux). Sama dengan yang dijelaskan di README root untuk pekerjaan latar (background jobs).
	</p>

	<p class="font-mono text-sm">
		Butuh gambaran besar? Buka
		<a class="text-gold hover:text-gold-bright" href={resolve('/docs/tools')}
			>Tools desktop dan editor</a
		>.
	</p>
</article>
