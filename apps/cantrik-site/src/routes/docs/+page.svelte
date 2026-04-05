<script lang="ts">
	import { resolve } from '$app/paths';
</script>

<main class="mx-auto max-w-3xl px-6 py-16">
	<h1 class="font-heading text-3xl font-semibold text-gold-bright">Documentation</h1>
	<p class="mt-4 font-mono text-sm text-ash">Placeholder: deep docs will live here.</p>
	<ul class="mt-8 list-inside list-disc font-mono text-sm text-ash">
		<li>
			<a
				class="text-gold hover:text-gold-bright"
				href="https://github.com/sangkan-dev/cantrik/blob/main/README.md"
				target="_blank"
				rel="noreferrer">README</a
			>
		</li>
		<li>
			<a
				class="text-gold hover:text-gold-bright"
				href="https://github.com/sangkan-dev/cantrik/blob/main/prd/cantrik-prd.md"
				target="_blank"
				rel="noreferrer">PRD</a
			>
		</li>
		<li>
			<a
				class="text-gold hover:text-gold-bright"
				href="https://github.com/sangkan-dev/cantrik/blob/main/CONTRIBUTING.md"
				target="_blank"
				rel="noreferrer">Contributing</a
			>
		</li>
		<li>
			<a class="text-gold hover:text-gold-bright" href={resolve('/docs/agent-harness')}
				>Agent harness (JSON / CI)</a
			>
		</li>
	</ul>
	<p class="mt-10 font-mono text-sm">
		<a class="text-gold hover:text-gold-bright" href={resolve('/')}>Back home</a>
	</p>
</main>
