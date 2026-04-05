<script lang="ts">
	import type { HTMLButtonAttributes } from 'svelte/elements';
	import { uiSoundTick } from '$lib/ui/sound';

	type Props = HTMLButtonAttributes & {
		children: import('svelte').Snippet;
	};

	let { children, onmouseenter, onfocus, type = 'button', ...rest }: Props = $props();

	function handleEnter(e: MouseEvent & { currentTarget: HTMLButtonElement }) {
		uiSoundTick();
		onmouseenter?.(e);
	}

	function handleFocus(e: FocusEvent & { currentTarget: HTMLButtonElement }) {
		uiSoundTick(1.05);
		onfocus?.(e);
	}
</script>

<button {type} {...rest} onmouseenter={handleEnter} onfocus={handleFocus}
	>{@render children()}</button
>
