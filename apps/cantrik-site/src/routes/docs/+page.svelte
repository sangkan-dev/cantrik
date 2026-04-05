<script lang="ts">
	import { resolve } from '$app/paths';
	import SoundLink from '$lib/components/SoundLink.svelte';
</script>

<svelte:head>
	<title>Dokumentasi - Cantrik (Sangkan)</title>
	<meta
		name="description"
		content="Mulai pakai Cantrik: install, konsep, tools desktop, keamanan, dan tautan ke spesifikasi teknis di repo."
	/>
</svelte:head>

<article
	class="prose prose-sm max-w-none prose-invert prose-headings:font-heading prose-headings:text-gold-bright prose-p:text-ash prose-a:text-gold prose-a:no-underline hover:prose-a:text-gold-bright prose-strong:text-[#e0e0e0] prose-li:text-ash"
>
	<h1>Dokumentasi Cantrik</h1>
	<p class="lead font-mono text-sm text-ash">
		<strong class="text-gold-dim">Satu kalimat:</strong> Cantrik membantu kamu di terminal dengan
		AI, tetap
		<strong>minta izin</strong> sebelum mengubah sistem.
	</p>

	<h2>Jalur belajar</h2>
	<ol class="list-decimal font-mono text-sm">
		<li>
			<SoundLink class="text-gold hover:text-gold-bright" href={resolve('/docs/start')}
				>Start (5 menit)</SoundLink
			>
			: install, satu perintah, kesan pertama.
		</li>
		<li>
			<SoundLink class="text-gold hover:text-gold-bright" href={resolve('/docs/concepts')}
				>Konsep</SoundLink
			>
			: CLI vs REPL, config global vs proyek, apa itu approve.
		</li>
		<li>
			<SoundLink class="text-gold hover:text-gold-bright" href={resolve('/docs/configure')}>
				Configure
			</SoundLink>
			: file config dan provider.
		</li>
		<li>
			<SoundLink class="text-gold hover:text-gold-bright" href={resolve('/docs/tools')}
				>Tray, Tauri, VS Code</SoundLink
			>
			: alat di luar terminal yang melengkapi CLI.
		</li>
		<li>
			<SoundLink class="text-gold hover:text-gold-bright" href={resolve('/docs/safety')}
				>Safety</SoundLink
			>: izin, sandbox, dan pola aman.
		</li>
	</ol>

	<h2>Detail teknis di repo</h2>
	<p class="font-mono text-sm">
		Situs ini pengantar. Spesifikasi panjang ada di
		<a
			class="text-gold hover:text-gold-bright"
			href="https://github.com/sangkan-dev/cantrik/blob/main/prd/cantrik-prd.md"
			target="_blank"
			rel="noreferrer">PRD</a
		>,
		<a
			class="text-gold hover:text-gold-bright"
			href="https://github.com/sangkan-dev/cantrik/blob/main/README.md"
			target="_blank"
			rel="noreferrer">README</a
		>, dan
		<a
			class="text-gold hover:text-gold-bright"
			href="https://github.com/sangkan-dev/cantrik/blob/main/CONTRIBUTING.md"
			target="_blank"
			rel="noreferrer">CONTRIBUTING</a
		>.
	</p>
</article>
