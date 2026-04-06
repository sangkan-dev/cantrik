<script lang="ts">
	import { resolve } from '$app/paths';
</script>

<svelte:head>
	<title>Peta ekosistem (rules, skills, plugin, MCP) - Dokumentasi Cantrik</title>
	<meta
		name="description"
		content="Ringkasan sederhana: rules, skills, plugin Lua/WASM, dan MCP di Cantrik — apa itu, di mana file-nya, cara pakai."
	/>
</svelte:head>

<article
	class="prose prose-sm max-w-none prose-invert prose-headings:font-heading prose-headings:text-gold-bright prose-p:text-ash prose-a:text-gold prose-strong:text-[#e0e0e0] prose-li:text-ash prose-table:text-sm"
>
	<h1>Peta ekosistem</h1>

	<p class="font-mono text-sm text-ash">
		<strong class="text-gold-dim">TL;DR:</strong> Cantrik bisa diperkaya dengan beberapa “lapisan”
		berbeda. Masing-masing punya tempat file dan cara aktifkan yang beda. Tabel di bawah
		membantu menjawab: <em>mau pakai apa?</em> → <em>file di mana?</em> → <em>langkah satu baris.</em>
	</p>

	<p class="font-mono text-sm text-ash">
		<strong class="text-gold-dim">Analogi sederhana:</strong> <strong>Rules</strong> seperti
		peraturan kelas untuk murid AI; <strong>skills</strong> seperti buku catatan pilihan yang
		kamu pinjam ke meja; <strong>plugin</strong> seperti tombol ekstra di mesin (Lua/WASM); <strong
			>MCP</strong
		>
		seperti colokan ke alat luar (database, filesystem server, dll.) lewat proses terpisah.
	</p>

	<div class="overflow-x-auto">
		<table class="w-full border-collapse font-mono text-xs">
			<thead>
				<tr class="border-b border-andesite-lighter text-left text-gold-dim">
					<th class="py-2 pr-4">Lapisan</th>
					<th class="py-2 pr-4">Apa</th>
					<th class="py-2 pr-4">Di mana (file)</th>
					<th class="py-2">Cara pakai (satu baris)</th>
				</tr>
			</thead>
			<tbody class="text-ash">
				<tr class="border-b border-andesite-lighter/60 align-top">
					<td class="py-3 pr-4 font-medium text-gold-bright">Rules</td>
					<td class="py-3 pr-4">Batas perilaku agen (instruksi proyek)</td>
					<td class="py-3 pr-4"><code>.cantrik/rules.md</code></td>
					<td class="py-3">Edit file itu lalu commit — Cantrik membacanya saat jalan di repo ini.</td>
				</tr>
				<tr class="border-b border-andesite-lighter/60 align-top">
					<td class="py-3 pr-4 font-medium text-gold-bright">Skills</td>
					<td class="py-3 pr-4">Konteks <code>.md</code> terpilih (paket skill)</td>
					<td class="py-3 pr-4"><code>.cantrik/skills/</code> + bagian <code>[skills]</code> di config</td>
					<td class="py-3">
						<code>cantrik skill install &lt;nama&gt;</code> dari registry lokal, atau salin manual.
					</td>
				</tr>
				<tr class="border-b border-andesite-lighter/60 align-top">
					<td class="py-3 pr-4 font-medium text-gold-bright">Plugin Lua / WASM</td>
					<td class="py-3 pr-4">Hook (mis. sebelum/sesudah write)</td>
					<td class="py-3 pr-4"><code>.cantrik/plugins/*.lua</code> atau <code>*.wasm</code></td>
					<td class="py-3">Letakkan file plugin + tinjau guardrails; tidak ada “install otomatis dari internet” di MVP.</td>
				</tr>
				<tr class="align-top">
					<td class="py-3 pr-4 font-medium text-gold-bright">MCP</td>
					<td class="py-3 pr-4">Alat eksternal (stdio server)</td>
					<td class="py-3 pr-4"><code>providers.toml</code> → <code>[mcp_client]</code></td>
					<td class="py-3">
						Tambah blok <code>[[mcp_client.servers]]</code>, lalu uji dengan
						<code>cantrik mcp call …</code>
					</td>
				</tr>
			</tbody>
		</table>
	</div>

	<h2>Hub registry</h2>
	<p class="font-mono text-sm text-ash">
		Di situs Cantrik ada daftar statis untuk contoh dan petunjuk singkat:
		<a class="text-gold hover:text-gold-bright" href={resolve('/registry')}>/registry</a>
		(<code>extensions.json</code>) dan
		<a class="text-gold hover:text-gold-bright" href={resolve('/registry/recipes')}>/registry/recipes</a>.
		CLI bisa menampilkan katalog yang sama: <code>cantrik registry list</code> dan
		<code>cantrik registry show &lt;id&gt;</code> (JSON disematkan di binary; opsi
		<code>--file</code> untuk uji file lain).
	</p>

	<p class="font-mono text-sm text-ash">
		<strong class="text-gold-dim">Skill siap pakai dari sumber:</strong> repo Cantrik menyertakan banyak paket di
		<code>contrib/skill-registry/</code> (review, MCP, tes Rust, dokumentasi, …). Salin folder ke
		<code>~/.local/share/cantrik/skill-registry/&lt;nama&gt;/</code> lalu
		<code>cantrik skill install &lt;nama&gt;</code> — daftar id mengikuti hub
		<a class="text-gold hover:text-gold-bright" href={resolve('/registry')}>/registry</a> (filter “Skill pack”).
	</p>

	<p class="font-mono text-sm text-ash">
		Ingin menambah entri? Lihat
		<a
			class="text-gold hover:text-gold-bright"
			href="https://github.com/sangkan-dev/cantrik/blob/main/CONTRIBUTING.md">CONTRIBUTING.md</a
		>
		bagian <em>Registry extensions</em>.
	</p>
</article>
