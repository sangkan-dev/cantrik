<script lang="ts">
	import { resolve } from '$app/paths';
	import type { ExtensionKind, RegistryExtension } from './+page';

	let { data } = $props();

	const kinds: (ExtensionKind | 'all')[] = [
		'all',
		'skill_pack',
		'lua_plugin',
		'wasm_plugin',
		'mcp_preset',
		'recipe_ref'
	];

	const kindLabel: Record<ExtensionKind | 'all', string> = {
		all: 'Semua',
		skill_pack: 'Skill pack',
		lua_plugin: 'Plugin Lua',
		wasm_plugin: 'Plugin WASM',
		mcp_preset: 'MCP preset',
		recipe_ref: 'Recipe'
	};

	let filterKind = $state<(ExtensionKind | 'all')>('all');
	let copiedId = $state<string | null>(null);

	const visible = $derived.by(() => {
		const list = data.extensions;
		if (filterKind === 'all') return list;
		return list.filter((e: RegistryExtension) => e.kind === filterKind);
	});

	async function copyHint(id: string, text: string) {
		try {
			await navigator.clipboard.writeText(text);
			copiedId = id;
			setTimeout(() => {
				copiedId = null;
			}, 2000);
		} catch {
			copiedId = null;
		}
	}
</script>

<svelte:head>
	<title>Extension registry — Cantrik</title>
	<meta
		name="description"
		content="Katalog statis skill pack, plugin, MCP preset, dan rujukan recipe Cantrik."
	/>
</svelte:head>

<main class="mx-auto max-w-3xl px-6 py-16">
	<h1 class="font-heading text-3xl font-semibold text-gold-bright">Extension registry</h1>
	<p class="mt-4 font-mono text-sm leading-relaxed text-ash">
		MVP: JSON statis di <code class="text-gold-dim">/registry/extensions.json</code>. Skema
		versi {data.schema_version}. Untuk penjelasan singkat tiap jenis (rules, skills, plugin,
		MCP), lihat
		<a
			class="text-gold underline decoration-gold/40 underline-offset-4 hover:text-gold-bright"
			href={resolve('/docs/extensions')}>dokumentasi peta ekosistem</a
		>. Recipes terpisah:
		<a
			class="text-gold underline decoration-gold/40 underline-offset-4 hover:text-gold-bright"
			href={resolve('/registry/recipes')}>/registry/recipes</a
		>.
	</p>
	<p class="mt-2 font-mono text-xs text-smoke">
		Legacy <code class="text-gold-dim">plugins.json</code> tetap ada untuk kompatibilitas; entri
		utama ada di <code class="text-gold-dim">extensions.json</code>.
	</p>

	<div class="mt-8 flex flex-wrap gap-2" role="toolbar" aria-label="Filter jenis">
		{#each kinds as k (k)}
			<button
				type="button"
				class="rounded border px-3 py-1.5 font-mono text-xs transition-colors {filterKind === k
					? 'border-gold bg-gold/10 text-gold-bright'
					: 'border-andesite-lighter bg-andesite-light text-ash hover:border-gold/40'}"
				aria-pressed={filterKind === k}
				onclick={() => (filterKind = k)}
			>
				{kindLabel[k]}
			</button>
		{/each}
	</div>

	{#if data.extensions.length === 0}
		<p class="mt-8 font-mono text-sm text-smoke">Belum ada entri.</p>
	{:else if visible.length === 0}
		<p class="mt-8 font-mono text-sm text-smoke">Tidak ada entri untuk filter ini.</p>
	{:else}
		<ul class="mt-10 space-y-6">
			{#each visible as e (e.id)}
				<li
					class="rounded border border-andesite-lighter bg-andesite-light px-4 py-4 font-mono text-sm"
				>
					<p class="font-heading text-base font-medium text-gold-bright">
						{e.name}
						{#if e.verified}
							<span
								class="ml-2 rounded bg-gold-dim/20 px-1.5 py-0.5 text-xs font-normal text-gold"
								title="Ditandai maintainer">verified</span
							>
						{/if}
					</p>
					<p class="mt-1 text-ash">{e.description}</p>
					<p class="mt-2 text-smoke">
						<span class="text-gold-dim">id</span>
						{e.id} · <span class="text-gold-dim">kind</span>
						{e.kind}
						{#if e.recipe_id}
							· <span class="text-gold-dim">recipe</span>
							{e.recipe_id}
						{/if}
					</p>
					<div class="mt-3 flex flex-wrap gap-2">
						<button
							type="button"
							class="rounded border border-gold/40 px-3 py-1 text-xs text-gold hover:bg-gold/10"
							onclick={() => copyHint(e.id, e.install_hint)}
						>
							{copiedId === e.id ? 'Disalin' : 'Salin petunjuk (install_hint)'}
						</button>
						<a
							class="rounded border border-andesite-lighter px-3 py-1 text-xs text-gold hover:border-gold/40"
							href={e.source}
							target="_blank"
							rel="noreferrer">Lihat sumber</a
						>
					</div>
					<pre
						class="mt-2 max-h-40 overflow-auto whitespace-pre-wrap rounded bg-andesite px-2 py-2 text-xs text-ash">{e.install_hint}</pre>
				</li>
			{/each}
		</ul>
	{/if}

	<p class="mt-10 flex flex-wrap gap-4 font-mono text-sm">
		<a class="text-gold hover:text-gold-bright" href={resolve('/registry/recipes')}>Recipes</a>
		<a class="text-gold hover:text-gold-bright" href={resolve('/docs/extensions')}>Docs ekosistem</a>
		<a class="text-gold hover:text-gold-bright" href={resolve('/')}>Home</a>
	</p>
</main>
