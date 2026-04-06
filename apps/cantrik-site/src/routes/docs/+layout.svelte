<script lang="ts">
	import { base, resolve } from '$app/paths';
	import { page } from '$app/state';
	import { docsNavTree, type DocNavEntry } from '$lib/docsNav';
	import SoundLink from '$lib/components/SoundLink.svelte';

	let { children } = $props();

	/** Internal docs paths; `resolve()` is strict per RouteId — base + pathname is correct for static hub. */
	function docHref(path: string) {
		const p = path.startsWith('/') ? path : `/${path}`;
		return `${base}${p}`;
	}

	const segments = $derived(page.url.pathname.split('/').filter(Boolean));

	const breadcrumbs = $derived.by(() => {
		const out: { href: string; label: string }[] = [{ href: resolve('/'), label: 'Home' }];
		let acc = '';
		for (let i = 0; i < segments.length; i++) {
			acc += `/${segments[i]}`;
			const seg = segments[i] ?? '';
			const label = breadcrumbLabel(seg, i);
			out.push({ href: docHref(acc), label });
		}
		return out;
	});

	function breadcrumbLabel(seg: string, index: number): string {
		const fixes: Record<string, string> = {
			docs: 'Docs',
			start: 'Start',
			concepts: 'Konsep',
			install: 'Install',
			configure: 'Configure',
			tools: 'Tools',
			tray: 'Tray',
			tauri: 'Tauri',
			vscode: 'VS Code',
			features: 'Fitur',
			session: 'Session memory',
			begawan: 'Begawan',
			'semantic-index': 'Semantic index',
			safety: 'Safety',
			extensions: 'Peta ekosistem',
			registry: 'Registry',
			'agent-harness': 'Agent harness'
		};
		if (fixes[seg]) return fixes[seg];
		if (index === 0) return seg;
		return seg
			.split('-')
			.map((w) => w.charAt(0).toUpperCase() + w.slice(1))
			.join(' ');
	}

	function normPath(path: string): string {
		const t = path.replace(/\/$/, '');
		return t || '/';
	}

	function navActive(entry: DocNavEntry): boolean {
		const p = normPath(page.url.pathname);
		const h = normPath(entry.href);
		if (p === h) return true;
		if (entry.children?.length) return p.startsWith(h + '/');
		return false;
	}
</script>

{#snippet navBranch(entries: DocNavEntry[], depth: number)}
	<ul class={depth === 0 ? 'space-y-1' : 'mt-1 space-y-1 border-l border-andesite-lighter pl-3'}>
		{#each entries as entry (entry.href)}
			<li>
				<SoundLink
					href={docHref(entry.href)}
					class="block rounded px-2 py-1 font-mono text-sm transition-colors {navActive(entry)
						? 'bg-andesite-lighter text-gold-bright'
						: 'text-ash hover:bg-andesite-light hover:text-gold'}"
				>
					{entry.label}
				</SoundLink>
				{#if entry.children?.length}
					{@render navBranch(entry.children, depth + 1)}
				{/if}
			</li>
		{/each}
	</ul>
{/snippet}

<div class="mx-auto flex max-w-6xl flex-col gap-8 px-6 py-8 md:flex-row md:py-12">
	<aside class="shrink-0 md:w-56 lg:w-64">
		<p class="font-heading text-xs font-medium tracking-wider text-gold-dim uppercase">
			Dokumentasi
		</p>
		<nav class="mt-4 max-h-[70vh] overflow-y-auto pr-2" aria-label="Dokumentasi Cantrik">
			{@render navBranch(docsNavTree, 0)}
		</nav>
	</aside>

	<div class="min-w-0 flex-1">
		<nav
			class="mb-8 flex flex-wrap items-center gap-1 font-mono text-xs text-smoke"
			aria-label="Breadcrumb"
		>
			{#each breadcrumbs as crumb, i (crumb.href + ':' + i)}
				{#if i > 0}
					<span class="text-andesite-lighter" aria-hidden="true">/</span>
				{/if}
				{#if i < breadcrumbs.length - 1}
					<SoundLink href={crumb.href} class="text-ash hover:text-gold">{crumb.label}</SoundLink>
				{:else}
					<span class="text-gold-dim">{crumb.label}</span>
				{/if}
			{/each}
		</nav>

		{@render children()}
	</div>
</div>
