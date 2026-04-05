<script lang="ts">
	import { resolve } from '$app/paths';

	type StatusShape = {
		sessions?: unknown[];
		jobs?: unknown[];
		[key: string]: unknown;
	};

	let paste = $state('');
	let parseError = $state<string | null>(null);
	let summary = $state<string | null>(null);

	function tryParse() {
		parseError = null;
		summary = null;
		const t = paste.trim();
		if (!t) {
			parseError = 'Paste JSON from cantrik status --json first.';
			return;
		}
		let data: StatusShape;
		try {
			data = JSON.parse(t) as StatusShape;
		} catch (e) {
			parseError = e instanceof Error ? e.message : 'Invalid JSON';
			return;
		}
		const sessions = Array.isArray(data.sessions) ? data.sessions.length : 0;
		const jobs = Array.isArray(data.jobs) ? data.jobs.length : 0;
		const keys = Object.keys(data).sort().join(', ');
		summary = `Top-level keys: ${keys || '(none)'}\nSessions (array length): ${sessions}\nJobs (array length): ${jobs}\n\nTip: run cantrik status --write-harness-summary for .cantrik/session-harness-summary.json.`;
	}
</script>

<main class="mx-auto max-w-3xl px-6 py-16">
	<h1 class="font-heading text-3xl font-semibold text-gold-bright">Local dashboard</h1>
	<p class="mt-4 font-mono text-sm leading-relaxed text-ash">
		The hub stays static; session-aware views come from the CLI. Run
		<code class="text-gold-dim">cantrik status --json</code>
		in your project. Use the VS Code Cantrik side panel webview (Status and registry) to render JSON in
		the editor.
	</p>
	<ul class="mt-8 list-inside list-disc space-y-3 font-mono text-sm text-ash">
		<li>
			<a class="text-gold hover:text-gold-bright" href={resolve('/registry')}>Plugin registry</a>
		</li>
		<li>
			<a class="text-gold hover:text-gold-bright" href={resolve('/registry/recipes')}>Recipe registry</a>
			— <code class="text-gold-dim">/registry/recipes.json</code>
		</li>
		<li>Multi-agent reviewer: <code class="text-gold-dim">cantrik agents "..." --reflect</code></li>
		<li>
			<a class="text-gold hover:text-gold-bright" href={resolve('/docs/agent-harness')}>Agent harness</a>
			— docs for harness summaries and re-plan.
		</li>
	</ul>

	<section class="mt-12 rounded border border-andesite-lighter bg-andesite-light p-4">
		<h2 class="font-heading text-lg font-medium text-gold-bright">Parse status JSON (browser-only)</h2>
		<p class="mt-2 font-mono text-xs leading-relaxed text-smoke">
			Paste output of <code class="text-gold-dim">cantrik status --json</code>. Nothing is uploaded; parsing runs
			only in this tab.
		</p>
		<textarea
			bind:value={paste}
			class="mt-3 h-40 w-full resize-y rounded border border-andesite-lighter bg-obsidian px-3 py-2 font-mono text-xs text-ash"
			placeholder="Paste output of cantrik status --json (object with sessions, jobs, …)"
			spellcheck="false"
		></textarea>
		<button
			type="button"
			class="mt-3 rounded bg-gold-dim/30 px-4 py-2 font-mono text-sm text-gold hover:bg-gold-dim/50"
			onclick={tryParse}
		>
			Parse
		</button>
		{#if parseError}
			<p class="mt-3 font-mono text-sm text-red-400/90">{parseError}</p>
		{/if}
		{#if summary}
			<pre class="mt-3 whitespace-pre-wrap font-mono text-xs leading-relaxed text-ash">{summary}</pre>
		{/if}
	</section>

	<p class="mt-10">
		<a class="font-mono text-sm text-gold hover:text-gold-bright" href={resolve('/')}>Back home</a>
	</p>
</main>
