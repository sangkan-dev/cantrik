<script lang="ts">
	import { resolve } from '$app/paths';
	import SoundLink from '$lib/components/SoundLink.svelte';
</script>

<svelte:head>
	<title>Tools desktop dan editor - Dokumentasi Cantrik</title>
	<meta
		name="description"
		content="Perbandingan CLI, cantrik-tray, cantrik-tauri, dan ekstensi VS Code; kapan memakai masing-masing."
	/>
</svelte:head>

<article
	class="prose prose-sm max-w-none prose-invert prose-headings:font-heading prose-headings:text-gold-bright prose-p:text-ash prose-a:text-gold prose-strong:text-[#e0e0e0] prose-li:text-ash"
>
	<h1>Tray, Tauri, VS Code</h1>

	<p class="font-mono text-sm text-ash">
		<strong class="text-gold-dim">TL;DR:</strong> <strong>CLI</strong> adalah otaknya. Tray/Tauri
		memberi
		<strong>notifikasi desktop</strong> saat ada permintaan persetujuan. <strong>VS Code</strong> memudahkan
		perintah dari editor.
	</p>

	<div class="not-prose my-8 overflow-x-auto">
		<table class="w-full border-collapse font-mono text-xs text-ash">
			<thead>
				<tr class="border-b border-andesite-lighter text-left text-gold-dim">
					<th class="py-2 pr-4 font-medium">Komponen</th>
					<th class="py-2 pr-4 font-medium">Apa</th>
					<th class="py-2 font-medium">Kapan dipakai</th>
				</tr>
			</thead>
			<tbody class="align-top">
				<tr class="border-b border-andesite-lighter/80">
					<td class="py-3 pr-4 font-medium text-gold-bright">CLI</td>
					<td class="py-3 pr-4">Otak utama: ask, plan, index, REPL, guardrail.</td>
					<td class="py-3">Terminal sebagai tempat kerja utama.</td>
				</tr>
				<tr class="border-b border-andesite-lighter/80">
					<td class="py-3 pr-4 font-medium text-gold-bright">cantrik-tray</td>
					<td class="py-3 pr-4"
						>Proses ringan yang memantau file flag approval; kirim notifikasi OS.</td
					>
					<td class="py-3">Kamu ingin ping desktop tanpa jendela tray native penuh.</td>
				</tr>
				<tr class="border-b border-andesite-lighter/80">
					<td class="py-3 pr-4 font-medium text-gold-bright">cantrik-tauri</td>
					<td class="py-3 pr-4">Aplikasi Tauri (tray native, MVP terbatas).</td>
					<td class="py-3">Alternatif tray dengan entry sistem bawaan Tauri.</td>
				</tr>
				<tr>
					<td class="py-3 pr-4 font-medium text-gold-bright">VS Code</td>
					<td class="py-3 pr-4">Ekstensi: doctor, harness, LSP opsional, buka hub.</td>
					<td class="py-3"
						>Workflow di IDE; tetap memanggil <code class="text-gold-bright">cantrik</code> di PATH.</td
					>
				</tr>
			</tbody>
		</table>
	</div>

	<h2>Alur singkat</h2>
	<p class="font-mono text-sm text-ash">
		CLI menulis flag saat butuh persetujuan; tray atau Tauri membaca flag yang sama (default path
		data OS).
	</p>
	<pre
		class="not-prose overflow-x-auto rounded border border-andesite-lighter bg-andesite-light/60 p-4 font-mono text-xs leading-relaxed text-ash"><code>  User ---------> cantrik (CLI)
    |                 ^
    |                 |
    v                 |
  VS Code ext --------+

  cantrik (CLI) ---- writes flag ----> cantrik-tray
                    \-------------> cantrik-tauri</code
		></pre>

	<h2>Halaman detail</h2>
	<ul class="font-mono text-sm">
		<li>
			<SoundLink class="text-gold" href={resolve('/docs/tools/tray')}>cantrik-tray</SoundLink>
		</li>
		<li>
			<SoundLink class="text-gold" href={resolve('/docs/tools/tauri')}>cantrik-tauri</SoundLink>
		</li>
		<li><SoundLink class="text-gold" href={resolve('/docs/tools/vscode')}>VS Code</SoundLink></li>
	</ul>
</article>
