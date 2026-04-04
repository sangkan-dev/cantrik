<script lang="ts">
	import { resolve } from '$app/paths';
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>Cantrik — AI CLI agent (Sangkan)</title>
	<meta
		name="description"
		content="Cantrik: open-source Rust CLI agent — semantic index, multi-provider LLM, session memory. Hub cantrik.sangkan.dev."
	/>
</svelte:head>

<header class="border-b border-andesite-lighter bg-andesite/80 backdrop-blur-sm">
	<div class="mx-auto flex max-w-3xl flex-wrap items-center justify-between gap-4 px-6 py-4">
		<a
			href={resolve('/')}
			class="font-heading text-lg font-semibold text-gold-bright hover:text-gold"
		>
			Cantrik
		</a>
		<nav class="flex flex-wrap gap-4 font-mono text-sm">
			<a class="text-ash hover:text-gold" href={resolve('/docs')}>Docs</a>
			<a class="text-ash hover:text-gold" href={resolve('/registry')}>Plugin registry</a>
			<a
				class="text-ash hover:text-gold"
				href="https://github.com/sangkan-dev/cantrik"
				target="_blank"
				rel="noreferrer">GitHub</a
			>
		</nav>
	</div>
</header>
{@render children()}
