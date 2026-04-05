<script lang="ts">
	import { resolve } from '$app/paths';
	import CipherReveal from '$lib/components/CipherReveal.svelte';
	import SoundLink from '$lib/components/SoundLink.svelte';
	import SoundButton from '$lib/components/SoundButton.svelte';
	import { uiSoundTick } from '$lib/ui/sound';
</script>

<svelte:head>
	<title>Cantrik - AI CLI agent open source (Sangkan)</title>
	<meta
		name="description"
		content="Cantrik: agen AI di terminal berbasis Rust dari Sangkan. Semantic index, multi-provider LLM, session memory, open source."
	/>
</svelte:head>

<main class="mx-auto max-w-3xl px-6 py-12 md:py-20">
	<p class="font-mono text-xs tracking-widest text-gold-dim uppercase">
		Sangkan / Ancient Cybernetics
	</p>
	<p class="mt-2 max-w-xl font-mono text-sm text-ash">
		<a
			class="text-gold hover:text-gold-bright"
			href="https://sangkan.dev"
			target="_blank"
			rel="noreferrer"
			onmouseenter={() => uiSoundTick()}>sangkan.dev</a
		>
		- foundry perangkat lunak. Cantrik adalah produk CLI di bawah payung ini.
	</p>

	<h1
		class="mt-8 font-heading text-4xl font-semibold tracking-tight text-gold-bright md:text-5xl lg:text-6xl"
	>
		Cantrik
	</h1>
	<div class="mt-3">
		<CipherReveal aksara="ꦕꦤ꧀ꦠꦿꦶꦏ꧀" latin="Cantrik (aksara Jawa)" />
	</div>
	<p class="mt-6 font-mono text-sm leading-relaxed text-ash md:text-base">
		<strong class="text-[#e0e0e0]">Agen AI di terminal</strong> berbasis Rust: indeks semantik lokal,
		beberapa penyedia LLM, REPL, dan memori sesi. 100% open source - kode bisa diaudit.
	</p>

	<section class="mt-14">
		<h2 class="font-heading text-lg font-medium text-[#e0e0e0]">The Arsenal</h2>
		<p class="mt-2 font-mono text-sm text-ash">
			Tiga pilar yang dipetakan ke Cantrik hari ini (bukan klaim untuk produk lain):
		</p>
		<ul class="mt-4 space-y-3 font-mono text-sm text-ash">
			<li>
				<strong class="text-gold-dim">AI agents / dev tooling</strong>
				- CLI <code class="text-gold-bright">cantrik</code>, indexing, plugin (Lua/WASM), harness
				untuk workflowmu.
			</li>
			<li>
				<strong class="text-gold-dim">Kontrol &amp; keamanan pola</strong>
				- alur approve, sandbox, dan guardrail (detail di
				<SoundLink class="text-gold" href={resolve('/docs/safety')}>Safety</SoundLink>).
			</li>
			<li>
				<strong class="text-gold-dim">Ekosistem Sangkan</strong>
				- hub ini, registry plugin, dan resep; produk keamanan siber terpisah akan dijelaskan di kanal
				Sangkan jika relevan.
			</li>
		</ul>
	</section>

	<section class="mt-12 rounded border border-andesite-lighter bg-andesite-light/30 p-6">
		<h2 class="font-heading text-lg font-medium text-gold-bright">Kenapa Cantrik?</h2>
		<p class="mt-3 font-mono text-sm leading-relaxed text-ash">
			Dalam narasi pewayangan, <strong class="text-[#e0e0e0]">Cantrik</strong> adalah murid yang
			membantu
			<strong class="text-[#e0e0e0]">Begawan</strong> (kamu) menyiapkan pekerjaan teknis tanpa
			melampaui izin. Di terminal artinya: bantu baca kode, rencanakan refactor, jalankan alur -
			dengan
			<strong class="text-[#e0e0e0]">persetujuan eksplisit</strong> untuk hal yang mengubah sistem,
			ritme
			<strong class="text-[#e0e0e0]">bare-metal</strong> dari Rust, dan komitmen
			<strong class="text-[#e0e0e0]">open source</strong>.
		</p>
	</section>

	<section class="mt-12 flex flex-wrap gap-3">
		<SoundButton
			class="inline-flex border border-gold/60 bg-andesite-light px-5 py-2.5 font-heading text-sm font-medium text-gold transition-colors hover:border-gold-bright hover:bg-andesite-lighter hover:text-gold-bright"
			onclick={() => {
				uiSoundTick(1.1);
				document.getElementById('install')?.scrollIntoView({ behavior: 'smooth' });
			}}
		>
			Install
		</SoundButton>
		<SoundLink
			href={resolve('/docs/start')}
			class="inline-flex border border-gold/60 bg-andesite-light px-5 py-2.5 font-heading text-sm font-medium text-gold transition-colors hover:border-gold-bright hover:bg-andesite-lighter hover:text-gold-bright"
		>
			Docs mulai di sini
		</SoundLink>
		<a
			class="inline-flex border border-andesite-lighter px-5 py-2.5 font-heading text-sm font-medium text-ash transition-colors hover:border-gold/40 hover:text-gold"
			href="https://github.com/sangkan-dev/cantrik"
			target="_blank"
			rel="noreferrer"
			onmouseenter={() => uiSoundTick()}>GitHub</a
		>
		<SoundLink
			href={resolve('/registry')}
			class="inline-flex border border-andesite-lighter px-5 py-2.5 font-heading text-sm font-medium text-ash transition-colors hover:border-gold/40 hover:text-gold"
		>
			Registry
		</SoundLink>
		<SoundLink
			href={resolve('/registry/recipes')}
			class="inline-flex border border-andesite-lighter px-5 py-2.5 font-heading text-sm font-medium text-ash transition-colors hover:border-gold/40 hover:text-gold"
		>
			Recipes
		</SoundLink>
	</section>

	<section class="mt-14" id="install">
		<h2 class="font-heading text-lg font-medium text-[#e0e0e0]">Install</h2>
		<ul class="mt-4 list-inside list-disc font-mono text-sm text-ash">
			<li>
				<strong class="text-gold-dim">Dari sumber:</strong>
				<code class="text-ash">cargo install --path crates/cantrik-cli</code> (repo root)
			</li>
			<li>
				<strong class="text-gold-dim">Binary releases:</strong>
				<a
					class="text-gold underline decoration-gold/40 underline-offset-4 hover:text-gold-bright"
					href="https://github.com/sangkan-dev/cantrik/releases"
					target="_blank"
					rel="noreferrer"
					onmouseenter={() => uiSoundTick()}>GitHub Releases</a
				>
			</li>
		</ul>
		<p class="mt-4 font-mono text-sm">
			<SoundLink class="text-gold hover:text-gold-bright" href={resolve('/docs/install')}>
				Panduan install lengkap
			</SoundLink>
		</p>
	</section>

	<footer class="mt-20 border-t border-andesite-lighter pt-8 font-mono text-xs text-smoke">
		<p>
			Dokumentasi resmi pengguna mulai dari
			<SoundLink class="text-gold hover:text-gold-bright" href={resolve('/docs')}
				>cantrik.sangkan.dev/docs</SoundLink
			>. Spesifikasi teknis panjang tetap di repositori (PRD).
		</p>
		<p class="mt-3">
			<a
				class="text-ash hover:text-gold"
				href="https://github.com/sangkan-dev/cantrik"
				target="_blank"
				rel="noreferrer"
				onmouseenter={() => uiSoundTick()}>GitHub</a
			>
			- MIT / komunitas terbuka.
		</p>
	</footer>
</main>
