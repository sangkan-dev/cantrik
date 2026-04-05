<script lang="ts">
	import type { HTMLAnchorAttributes } from 'svelte/elements';
	import { uiSoundTick } from '$lib/ui/sound';

	type Props = HTMLAnchorAttributes & {
		children: import('svelte').Snippet;
	};

	let { children, onmouseenter, onfocus, ...rest }: Props = $props();

	function handleEnter(e: MouseEvent & { currentTarget: HTMLAnchorElement }) {
		uiSoundTick();
		onmouseenter?.(e);
	}

	function handleFocus(e: FocusEvent & { currentTarget: HTMLAnchorElement }) {
		uiSoundTick(1.05);
		onfocus?.(e);
	}
</script>

<a {...rest} onmouseenter={handleEnter} onfocus={handleFocus}>{@render children()}</a>
