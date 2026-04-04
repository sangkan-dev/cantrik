<script lang="ts">
	import { resolve } from '$app/paths';
</script>

<main class="mx-auto max-w-3xl px-6 py-16">
	<p class="font-mono text-sm tracking-widest text-gold-dim uppercase">
		Sangkan / Ancient Cybernetics
	</p>
	<h1 class="mt-2 font-heading text-4xl font-semibold tracking-tight text-gold-bright md:text-5xl">
		Cantrik
	</h1>
	<p class="mt-2 font-mono text-sm text-rust">Javanese logogram: Cantrik (see README)</p>
	<p class="mt-6 font-mono leading-relaxed text-ash">
		Open-source <strong class="text-[#e0e0e0]">AI CLI agent</strong> in Rust: semantic index, local embeddings,
		multi-provider LLM, REPL, and session memory.
	</p>

	<section class="mt-12">
		<h2 class="font-heading text-lg font-medium text-[#e0e0e0]">Install</h2>
		<ul class="mt-4 list-inside list-disc font-mono text-sm text-ash">
			<li>
				<strong class="text-gold-dim">From source:</strong>
				<code class="text-ash">cargo install --path crates/cantrik-cli</code> (repo root)
			</li>
			<li>
				<strong class="text-gold-dim">Binary releases:</strong>
				<a
					class="text-gold underline decoration-gold/40 underline-offset-4 hover:text-gold-bright"
					href="https://github.com/sangkan-dev/cantrik/releases">GitHub Releases</a
				>
			</li>
		</ul>
	</section>

	<section class="mt-12 flex flex-wrap gap-4">
		<a
			href={resolve('/docs')}
			class="inline-flex border border-gold/60 bg-andesite-light px-5 py-2.5 font-heading text-sm font-medium text-gold transition-colors hover:border-gold-bright hover:bg-andesite-lighter hover:text-gold-bright"
		>
			Documentation
		</a>
		<a
			href={resolve('/registry')}
			class="inline-flex border border-andesite-lighter px-5 py-2.5 font-heading text-sm font-medium text-ash transition-colors hover:border-gold/40 hover:text-gold"
		>
			Plugin registry (MVP)
		</a>
		<a
			href="https://sangkan.dev"
			class="inline-flex border border-andesite-lighter px-5 py-2.5 font-mono text-sm text-smoke transition-colors hover:text-ash"
		>
			Sangkan forge
		</a>
	</section>

	<p class="mt-14 font-mono text-xs text-smoke">
		Hub: cantrik.sangkan.dev — PRD alias cantrik.dev may redirect when configured.
	</p>
</main>
