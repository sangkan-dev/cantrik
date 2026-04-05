<script lang="ts">
	import { resolve } from '$app/paths';

	let { data } = $props();
</script>

<main class="mx-auto max-w-3xl px-6 py-16">
	<h1 class="font-heading text-3xl font-semibold text-gold-bright">Recipe registry</h1>
	<p class="mt-4 font-mono text-sm leading-relaxed text-ash">
		Static JSON at <code class="text-gold-dim">/registry/recipes.json</code>. Aligns with
		<code class="text-gold-dim">cantrik init --template &lt;name&gt;</code> when
		<code class="text-gold-dim">init_template</code> matches. Schema version {data.schema_version}.
	</p>

	{#if data.recipes.length === 0}
		<p class="mt-8 font-mono text-sm text-smoke">No recipes listed yet.</p>
	{:else}
		<ul class="mt-10 space-y-6">
			{#each data.recipes as r (r.id)}
				<li
					class="rounded border border-andesite-lighter bg-andesite-light px-4 py-4 font-mono text-sm"
				>
					<p class="font-heading text-base font-medium text-gold-bright">{r.title}</p>
					<p class="mt-1 text-ash">{r.description}</p>
					<p class="mt-2 text-smoke">
						<span class="text-gold-dim">id</span> {r.id} ·
						<span class="text-gold-dim">init</span>
						<code class="text-gold">cantrik init --template {r.init_template}</code>
					</p>
				</li>
			{/each}
		</ul>
	{/if}

	<p class="mt-10 flex flex-wrap gap-4 font-mono text-sm">
		<a class="text-gold hover:text-gold-bright" href={resolve('/registry')}>Plugin registry</a>
		<a class="text-gold hover:text-gold-bright" href={resolve('/dashboard')}>Dashboard</a>
		<a class="text-gold hover:text-gold-bright" href={resolve('/')}>Home</a>
	</p>
</main>
