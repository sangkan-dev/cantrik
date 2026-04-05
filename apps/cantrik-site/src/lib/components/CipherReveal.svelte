<script lang="ts">
	import { browser } from '$app/environment';
	import { prefersReducedMotion } from '$lib/ui/sound';

	type Props = {
		aksara: string;
		latin: string;
		class?: string;
	};

	let { aksara, latin, class: className = '' }: Props = $props();

	let root: HTMLElement | undefined = $state();
	let revealed = $state(false);

	$effect(() => {
		if (!browser || !root) return;
		if (prefersReducedMotion()) {
			revealed = true;
			return;
		}
		const el = root;
		const obs = new IntersectionObserver(
			(entries) => {
				for (const e of entries) {
					if (e.isIntersecting) {
						revealed = true;
						obs.disconnect();
						break;
					}
				}
			},
			{ rootMargin: '0px 0px -10% 0px', threshold: 0.2 }
		);
		obs.observe(el);
		return () => obs.disconnect();
	});
</script>

<span
	bind:this={root}
	class="cipher-reveal inline-flex flex-col items-start gap-0 sm:inline-flex sm:flex-row sm:items-baseline sm:gap-3 {className}"
	aria-label="{latin} ({aksara})"
>
	<span
		class="cipher-reveal__aksara text-gold-bright"
		style="font-family: 'Noto Sans Javanese', sans-serif; line-height: 1.6;"
		lang="jv">{aksara}</span
	>
	<span
		class="cipher-reveal__latin font-mono text-sm text-rust transition-opacity duration-700 ease-out"
		class:opacity-0={!revealed}
		class:opacity-100={revealed}
		aria-hidden="true">{latin}</span
	>
</span>

<style>
	@media (prefers-reduced-motion: reduce) {
		.cipher-reveal__latin {
			opacity: 1 !important;
			transition: none;
		}
		.cipher-reveal__aksara {
			opacity: 0.85;
		}
	}
</style>
