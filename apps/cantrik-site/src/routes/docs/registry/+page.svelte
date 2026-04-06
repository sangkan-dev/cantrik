<script lang="ts">
	import { resolve } from '$app/paths';
</script>

<svelte:head>
	<title>Registry dan recipes - Dokumentasi Cantrik</title>
	<meta
		name="description"
		content="Extension registry, recipes, dan peta ekosistem Cantrik di dokumentasi hub."
	/>
</svelte:head>

<article
	class="prose prose-sm max-w-none prose-invert prose-headings:font-heading prose-headings:text-gold-bright prose-p:text-ash prose-a:text-gold prose-strong:text-[#e0e0e0] prose-li:text-ash"
>
	<h1>Registry dan recipes</h1>

	<p class="font-mono text-sm text-ash">
		<strong class="text-gold-dim">Satu kalimat:</strong> hub punya
		<strong>katalog ekstensi</strong> (skill pack, plugin, MCP preset, rujukan recipe), plus
		<strong>recipes</strong> untuk <code>cantrik init</code>.
	</p>

	<ul class="font-mono text-sm">
		<li>
			<a class="text-gold hover:text-gold-bright" href={resolve('/docs/extensions')}
				>Peta ekosistem (rules / skills / plugin / MCP)</a
			>
		</li>
		<li>
			<a class="text-gold hover:text-gold-bright" href={resolve('/registry')}>Extension registry</a>
		</li>
		<li>
			<a class="text-gold hover:text-gold-bright" href={resolve('/registry/recipes')}>Recipes</a>
		</li>
	</ul>

	<p class="text-sm">
		Spesifikasi makro dan recipe system ada di PRD sekitar <strong
			>4.18 Macro and Recipe System</strong
		> dan bagian plugin.
	</p>
</article>
