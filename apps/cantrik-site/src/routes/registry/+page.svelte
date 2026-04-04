<script lang="ts">
	import { resolve } from '$app/paths';

	let { data } = $props();
</script>

<main class="mx-auto max-w-3xl px-6 py-16">
	<h1 class="font-heading text-3xl font-semibold text-gold-bright">Plugin registry</h1>
	<p class="mt-4 font-mono text-sm leading-relaxed text-ash">
		MVP: static JSON (<code class="text-gold-dim">/registry/plugins.json</code>). Schema version
		{data.schema_version}. Listing will expand with community submissions.
	</p>

	{#if data.plugins.length === 0}
		<p class="mt-8 font-mono text-sm text-smoke">No plugins listed yet.</p>
	{:else}
		<ul class="mt-10 space-y-6">
			{#each data.plugins as p (p.id)}
				<li
					class="rounded border border-andesite-lighter bg-andesite-light px-4 py-4 font-mono text-sm"
				>
					<p class="font-heading text-base font-medium text-gold-bright">{p.name}</p>
					<p class="mt-1 text-ash">{p.description}</p>
					<a
						class="mt-2 inline-block text-gold underline decoration-gold/40 underline-offset-4 hover:text-gold-bright"
						href={p.repo}
						target="_blank"
						rel="noreferrer">{p.repo}</a
					>
				</li>
			{/each}
		</ul>
	{/if}

	<p class="mt-10">
		<a class="font-mono text-sm text-gold hover:text-gold-bright" href={resolve('/')}>Back home</a>
	</p>
</main>
