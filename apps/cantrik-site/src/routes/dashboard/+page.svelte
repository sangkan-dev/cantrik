<script lang="ts">
	import { resolve } from '$app/paths';
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
	</ul>
	<p class="mt-10">
		<a class="font-mono text-sm text-gold hover:text-gold-bright" href={resolve('/')}>Back home</a>
	</p>
</main>
