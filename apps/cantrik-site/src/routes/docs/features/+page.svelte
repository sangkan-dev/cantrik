<script lang="ts">
	import { resolve } from '$app/paths';
	import SoundLink from '$lib/components/SoundLink.svelte';
</script>

<svelte:head>
	<title>Fitur - Dokumentasi Cantrik</title>
	<meta
		name="description"
		content="Peta fitur Cantrik: semantic index, session memory, Begawan - ringkas dengan tautan ke PRD."
	/>
</svelte:head>

<article
	class="prose prose-sm max-w-none prose-invert prose-headings:font-heading prose-headings:text-gold-bright prose-p:text-ash prose-a:text-gold prose-strong:text-[#e0e0e0] prose-li:text-ash"
>
	<h1>Fitur (ringkas)</h1>

	<p class="font-mono text-sm text-ash">
		Halaman ini memetakan area besar PRD ke penjelasan singkat. Untuk dalil implementasi, buka
		<a
			class="text-gold hover:text-gold-bright"
			href="https://github.com/sangkan-dev/cantrik/blob/main/prd/cantrik-prd.md"
			target="_blank"
			rel="noreferrer">cantrik-prd.md</a
		>
		bagian 4.
	</p>

	<h2>Topik di situs ini</h2>
	<ul class="font-mono text-sm">
		<li>
			<SoundLink class="text-gold" href={resolve('/docs/features/semantic-index')}>
				Semantic index / codebase intelligence
			</SoundLink>
		</li>
		<li>
			<SoundLink class="text-gold" href={resolve('/docs/features/session')}
				>Session memory</SoundLink
			>
		</li>
		<li>
			<SoundLink class="text-gold" href={resolve('/docs/features/begawan')}
				>Begawan dan multi-agent</SoundLink
			>
		</li>
	</ul>

	<h2>Lainnya di PRD (satu baris)</h2>
	<ul class="text-sm">
		<li><strong>Background mode</strong> - agen jalan di belakang; tray membantu approval.</li>
		<li><strong>Sandbox</strong> - eksekusi terbatas; lihat Safety.</li>
		<li><strong>MCP, plugin Lua/WASM, registry</strong> - ekosistem perluasan.</li>
		<li>
			<strong>Extension registry</strong> di hub: <SoundLink
				class="text-gold"
				href={resolve('/registry')}>/registry</SoundLink
			>.
		</li>
	</ul>
</article>
