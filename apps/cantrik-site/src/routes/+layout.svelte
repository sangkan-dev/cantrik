<script lang="ts">
	import { resolve } from '$app/paths';
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import SoundLink from '$lib/components/SoundLink.svelte';
	import SoundButton from '$lib/components/SoundButton.svelte';
	import { isAudioMuted } from '$lib/sangkan/audio';
	import { uiSoundTick } from '$lib/ui/sound';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>Cantrik - AI CLI agent (Sangkan)</title>
	<meta
		name="description"
		content="Cantrik: open-source Rust CLI agent - semantic index, multi-provider LLM, session memory. Dokumentasi di cantrik.sangkan.dev/docs."
	/>
</svelte:head>

<header class="border-b border-andesite-lighter bg-andesite/80 backdrop-blur-sm">
	<div class="mx-auto flex max-w-6xl flex-wrap items-center justify-between gap-4 px-6 py-4">
		<div class="flex flex-wrap items-center gap-4">
			<SoundLink
				href={resolve('/')}
				class="font-heading text-lg font-semibold text-gold-bright hover:text-gold"
			>
				Cantrik
			</SoundLink>
			<span class="hidden font-mono text-xs text-smoke sm:inline" aria-hidden="true">|</span>
			<a
				class="hidden font-mono text-xs text-smoke hover:text-ash sm:inline"
				href="https://sangkan.dev"
				target="_blank"
				rel="noreferrer"
				onmouseenter={() => uiSoundTick()}>Sangkan</a
			>
		</div>
		<div class="flex flex-wrap items-center gap-3">
			<nav class="flex flex-wrap gap-x-4 gap-y-2 font-mono text-sm" aria-label="Utama">
				<SoundLink class="text-ash hover:text-gold" href={resolve('/docs')}>Docs</SoundLink>
				<SoundLink class="text-ash hover:text-gold" href={resolve('/docs/start')}>Start</SoundLink>
				<SoundLink class="text-ash hover:text-gold" href={resolve('/docs/tools')}>Tools</SoundLink>
				<SoundLink class="text-ash hover:text-gold" href={resolve('/docs/safety')}>Safety</SoundLink
				>
				<SoundLink class="text-ash hover:text-gold" href={resolve('/registry')}>Registry</SoundLink>
				<SoundLink class="text-ash hover:text-gold" href={resolve('/registry/recipes')}
					>Recipes</SoundLink
				>
				<SoundLink class="text-ash hover:text-gold" href={resolve('/dashboard')}
					>Dashboard</SoundLink
				>
				<SoundLink class="text-ash hover:text-gold" href={resolve('/docs/agent-harness')}
					>Harness</SoundLink
				>
				<a
					class="text-ash hover:text-gold"
					href="https://github.com/sangkan-dev/cantrik"
					target="_blank"
					rel="noreferrer"
					onmouseenter={() => uiSoundTick()}>GitHub</a
				>
			</nav>
			<SoundButton
				type="button"
				class="rounded border border-andesite-lighter px-2 py-1 font-mono text-xs text-ash hover:border-gold/40 hover:text-gold"
				aria-label={$isAudioMuted ? 'Nyalakan suara UI' : 'Matikan suara UI'}
				aria-pressed={$isAudioMuted}
				onclick={() => {
					isAudioMuted.update((m) => !m);
					uiSoundTick(0.95);
				}}
			>
				{$isAudioMuted ? 'Unmute' : 'Mute'}
			</SoundButton>
		</div>
	</div>
</header>
{@render children()}
