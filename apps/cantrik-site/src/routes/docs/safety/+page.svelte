<script lang="ts">
	import { resolve } from '$app/paths';
	import SoundLink from '$lib/components/SoundLink.svelte';
</script>

<svelte:head>
	<title>Safety dan approve - Dokumentasi Cantrik</title>
	<meta
		name="description"
		content="Approve, sandbox, fetch: guardrail Cantrik dalam bahasa sederhana."
	/>
</svelte:head>

<article
	class="prose prose-sm max-w-none prose-invert prose-headings:font-heading prose-headings:text-gold-bright prose-p:text-ash prose-a:text-gold prose-strong:text-[#e0e0e0] prose-code:text-gold-bright prose-li:text-ash"
>
	<h1>Safety dan approve</h1>

	<p class="font-mono text-sm text-ash">
		<strong class="text-gold-dim">TL;DR:</strong> Cantrik dirancang agar kamu bisa melihat dan
		<strong>menyetujui</strong> langkah berisiko, bukan diam-diam mengubah sistem.
	</p>

	<aside
		class="not-prose my-6 rounded border border-gold/30 bg-andesite-light/40 p-4 font-mono text-sm text-ash"
	>
		<p class="font-heading text-sm font-medium text-gold-dim">Analogi</p>
		<p class="mt-2">
			Seperti dialog &quot;Yakin ingin menghapus?&quot; - hanya untuk tugas yang bisa merusak atau
			mengubah file penting.
		</p>
	</aside>

	<h2>Approve</h2>
	<p>
		Alur persetujuan bisa berupa prompt di terminal atau sinyal lewat file flag yang dibaca
		<SoundLink class="text-gold" href={resolve('/docs/tools')}>tray/Tauri</SoundLink> - tergantung setupmu.
	</p>

	<h2>Sandbox</h2>
	<p>
		Eksekusi perintah atau tool bisa dibatasi (sandbox) supaya dampaknya terkendali. Detail opsi
		lanjutan ada di PRD bagian sandbox.
	</p>

	<h2>Fetch dan jaringan</h2>
	<p>
		Tool yang menyentuh jaringan atau lingkungan eksternal sebaiknya kamu pahami sebelum disetujui -
		sama seperti memberi izin aplikasi lain.
	</p>

	<p class="font-mono text-sm">
		Dalil lengkap: PRD <strong>5. Guardrails</strong> dan <strong>Permission tiers</strong>.
	</p>
</article>
