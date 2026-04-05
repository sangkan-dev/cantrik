import { browser } from '$app/environment';
import { get } from 'svelte/store';
import { cyberAudio, isAudioMuted } from '$lib/sangkan/audio';

export function prefersReducedMotion(): boolean {
	if (!browser) return true;
	return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

/** Short UI tick on hover/focus; skips when muted or reduced motion. */
export function uiSoundTick(pitch = 1): void {
	if (!browser || prefersReducedMotion()) return;
	cyberAudio.init();
	if (!get(isAudioMuted)) cyberAudio.playTick(pitch);
}
