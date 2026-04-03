const {
  Document, Packer, Paragraph, TextRun, Table, TableRow, TableCell,
  Header, Footer, AlignmentType, HeadingLevel, BorderStyle, WidthType,
  ShadingType, VerticalAlign, PageBreak, LevelFormat,
  ExternalHyperlink, TabStopType, TabStopPosition,
  SimpleField,
} = require('docx');
const fs = require('fs');

// ── Color palette ──────────────────────────────────────────────
const C = {
  gold:    "C9A84C",
  rust:    "C0540A",
  dark:    "1A1A2E",
  smoke:   "555566",
  light:   "F5F0E8",
  white:   "FFFFFF",
  border:  "C9A84C",
  accent:  "2D4A7A",
  tableHd: "1A1A2E",
  tableRw: "F5F0E8",
  tableA:  "EDE8DE",
};

// ── Helpers ────────────────────────────────────────────────────
const gold  = (text, opts={}) => new TextRun({ text, color: C.gold, font: "Consolas", ...opts });
const bold  = (text, opts={}) => new TextRun({ text, bold: true, ...opts });
const norm  = (text, opts={}) => new TextRun({ text, ...opts });
const smoke = (text, opts={}) => new TextRun({ text, color: C.smoke, italics: true, ...opts });
const code  = (text, opts={}) => new TextRun({ text, font: "Consolas", color: C.rust, size: 20, ...opts });

const spacer = (n=1) => Array.from({length:n}, () =>
  new Paragraph({ children: [new TextRun("")], spacing: { after: 0 } })
);

const hr = () => new Paragraph({
  children: [new TextRun("")],
  border: { bottom: { style: BorderStyle.SINGLE, size: 6, color: C.gold, space: 1 } },
  spacing: { before: 160, after: 160 },
});

const h1 = (text) => new Paragraph({
  heading: HeadingLevel.HEADING_1,
  children: [new TextRun({ text, bold: true, font: "Arial", size: 36, color: C.dark })],
  spacing: { before: 480, after: 200 },
  border: { bottom: { style: BorderStyle.SINGLE, size: 8, color: C.gold, space: 4 } },
});

const h2 = (text) => new Paragraph({
  heading: HeadingLevel.HEADING_2,
  children: [new TextRun({ text, bold: true, font: "Arial", size: 28, color: C.accent })],
  spacing: { before: 360, after: 160 },
});

const h3 = (text) => new Paragraph({
  heading: HeadingLevel.HEADING_3,
  children: [new TextRun({ text, bold: true, font: "Arial", size: 24, color: C.rust })],
  spacing: { before: 240, after: 120 },
});

const p = (children, opts={}) => new Paragraph({
  children: Array.isArray(children) ? children : [norm(children)],
  spacing: { after: 140 },
  ...opts,
});

const bullet = (children, level=0) => new Paragraph({
  numbering: { reference: "bullets", level },
  children: Array.isArray(children) ? children : [norm(children)],
  spacing: { after: 80 },
});

const num = (children, ref="numbers") => new Paragraph({
  numbering: { reference: ref, level: 0 },
  children: Array.isArray(children) ? children : [norm(children)],
  spacing: { after: 100 },
});

const codeBlock = (lines) => lines.map(line =>
  new Paragraph({
    children: [new TextRun({ text: line, font: "Consolas", size: 18, color: C.dark })],
    spacing: { after: 0, line: 240 },
    indent: { left: 720 },
    shading: { fill: "F0EDE5", type: ShadingType.CLEAR },
  })
);

const bdr = { style: BorderStyle.SINGLE, size: 1, color: C.border };
const borders = { top: bdr, bottom: bdr, left: bdr, right: bdr };

const thCell = (text, w) => new TableCell({
  borders,
  width: { size: w, type: WidthType.DXA },
  shading: { fill: C.tableHd, type: ShadingType.CLEAR },
  margins: { top: 100, bottom: 100, left: 160, right: 160 },
  children: [new Paragraph({
    children: [new TextRun({ text, bold: true, color: C.white, font: "Arial", size: 20 })],
  })],
});

const tdCell = (text, w, shade=C.white, italic=false, color=C.dark) => new TableCell({
  borders,
  width: { size: w, type: WidthType.DXA },
  shading: { fill: shade, type: ShadingType.CLEAR },
  margins: { top: 80, bottom: 80, left: 160, right: 160 },
  children: [new Paragraph({
    children: [new TextRun({ text, font: "Arial", size: 20, italics: italic, color })],
  })],
});

const tdCode = (text, w, shade=C.white) => new TableCell({
  borders,
  width: { size: w, type: WidthType.DXA },
  shading: { fill: shade, type: ShadingType.CLEAR },
  margins: { top: 80, bottom: 80, left: 160, right: 160 },
  children: [new Paragraph({
    children: [new TextRun({ text, font: "Consolas", size: 18, color: C.rust })],
  })],
});

// ── Cover page ─────────────────────────────────────────────────
const coverPage = [
  ...spacer(6),
  new Paragraph({
    alignment: AlignmentType.CENTER,
    children: [new TextRun({ text: "ꦕꦤ꧀ꦠꦿꦶꦏ꧀", font: "Noto Serif Javanese", size: 72, color: C.gold, bold: true })],
    spacing: { after: 120 },
  }),
  new Paragraph({
    alignment: AlignmentType.CENTER,
    children: [new TextRun({ text: "C A N T R I K", font: "Arial", size: 52, bold: true, color: C.dark, characterSpacing: 300 })],
    spacing: { after: 80 },
  }),
  new Paragraph({
    alignment: AlignmentType.CENTER,
    children: [new TextRun({ text: "Open-Source AI CLI Agent", font: "Arial", size: 28, color: C.smoke, italics: true })],
    spacing: { after: 480 },
  }),
  new Paragraph({
    alignment: AlignmentType.CENTER,
    border: { bottom: { style: BorderStyle.SINGLE, size: 4, color: C.gold, space: 1 }, top: { style: BorderStyle.SINGLE, size: 4, color: C.gold, space: 1 } },
    children: [new TextRun({ text: "Product Requirements Document  ·  Technical Specification  ·  Roadmap", font: "Arial", size: 22, color: C.accent })],
    spacing: { before: 80, after: 80 },
  }),
  ...spacer(8),
  new Paragraph({
    alignment: AlignmentType.CENTER,
    children: [new TextRun({ text: "Version 1.0  ·  2025", font: "Arial", size: 20, color: C.smoke })],
  }),
  new Paragraph({ children: [new PageBreak()] }),
];

// ── Filosofi page ──────────────────────────────────────────────
const filosofiPage = [
  h1("Filosofi & Visi"),
  p([
    norm("Dalam pewayangan dan budaya Jawa kuno, "),
    bold("Cantrik"),
    norm(" adalah seorang murid sekaligus asisten setia dari seorang "),
    bold("Begawan"),
    norm(" — guru besar atau pertapa yang bijaksana. Seorang Cantrik bertugas menyiapkan segala keperluan teknis, membersihkan jalan, dan membantu sang guru mewujudkan pemikirannya "),
    italic_r("tanpa pernah melampaui kehendak gurunya."),
  ]),
  spacer(1)[0],
  h2("Makna Teknis"),
  p([
    norm("Di terminal, Cantrik adalah CLI Agent yang menyiapkan boilerplate, membaca konteks kode, menjalankan script, dan mencari bug — sehingga "),
    bold("kamu (Sang Begawan)"),
    norm(" bisa fokus pada arsitektur tingkat tinggi."),
  ]),
  spacer(1)[0],
  h2("Tiga Pilar Utama"),
  bullet([bold("Etika Tinggi. "), norm("Cantrik tidak pernah bertindak tanpa persetujuan Begawan. Setiap aksi yang memodifikasi sistem memerlukan approval eksplisit.")]),
  bullet([bold("Bare-metal Fast. "), norm("Dibangun di atas Rust dengan async tokio — tidak ada lag, tidak ada overhead, tidak ada runtime berat.")]),
  bullet([bold("100% Open Source. "), norm("Cantrik bukan wrapper API biasa. Setiap baris kodenya terbuka, dapat diaudit, dan dapat dikontribusi.")]),
  spacer(1)[0],
  h2("Perbedaan dengan Tools Lain"),
  new Table({
    width: { size: 9360, type: WidthType.DXA },
    columnWidths: [2600, 2253, 2253, 2254],
    rows: [
      new TableRow({ children: [thCell("Aspek", 2600), thCell("Claude Code", 2253), thCell("Gemini CLI", 2253), thCell("Cantrik", 2254)] }),
      new TableRow({ children: [tdCell("Source", 2600, C.tableRw), tdCell("Closed (leaked)", 2253, C.tableRw), tdCell("Open Source", 2253, C.tableRw), tdCell("Open Source ✓", 2254, C.tableRw, false, C.rust)] }),
      new TableRow({ children: [tdCell("Core Language", 2600, C.tableA), tdCell("TypeScript", 2253, C.tableA), tdCell("TypeScript", 2253, C.tableA), tdCell("Rust ✓", 2254, C.tableA, false, C.rust)] }),
      new TableRow({ children: [tdCell("Multi-provider LLM", 2600, C.tableRw), tdCell("Claude only", 2253, C.tableRw), tdCell("Gemini only", 2253, C.tableRw), tdCell("Plug-and-play ✓", 2254, C.tableRw, false, C.rust)] }),
      new TableRow({ children: [tdCell("Vector Memory", 2600, C.tableA), tdCell("Limited", 2253, C.tableA), tdCell("Limited", 2253, C.tableA), tdCell("Native ✓", 2254, C.tableA, false, C.rust)] }),
      new TableRow({ children: [tdCell("Plugin System", 2600, C.tableRw), tdCell("Partial", 2253, C.tableRw), tdCell("Partial", 2253, C.tableRw), tdCell("Lua + WASM ✓", 2254, C.tableRw, false, C.rust)] }),
      new TableRow({ children: [tdCell("Multi-Agent", 2600, C.tableA), tdCell("In Dev", 2253, C.tableA), tdCell("Limited", 2253, C.tableA), tdCell("Native ✓", 2254, C.tableA, false, C.rust)] }),
      new TableRow({ children: [tdCell("Offline Support", 2600, C.tableRw), tdCell("No", 2253, C.tableRw), tdCell("Partial", 2253, C.tableRw), tdCell("Full (Ollama) ✓", 2254, C.tableRw, false, C.rust)] }),
    ],
  }),
  new Paragraph({ children: [new PageBreak()] }),
];

function italic_r(text) { return new TextRun({ text, italics: true }); }

// ── Architecture ───────────────────────────────────────────────
const architecturePage = [
  h1("Arsitektur Sistem"),
  h2("Stack Teknologi"),
  new Table({
    width: { size: 9360, type: WidthType.DXA },
    columnWidths: [2800, 3280, 3280],
    rows: [
      new TableRow({ children: [thCell("Layer", 2800), thCell("Teknologi", 3280), thCell("Alasan", 3280)] }),
      new TableRow({ children: [tdCell("Core Engine", 2800, C.tableRw), tdCode("Rust + tokio", 3280, C.tableRw), tdCell("Async, memory-safe, zero-cost abstraction", 3280, C.tableRw)] }),
      new TableRow({ children: [tdCell("CLI Framework", 2800, C.tableA), tdCode("clap v4", 3280, C.tableA), tdCell("Mature, derive macro, shell completion", 3280, C.tableA)] }),
      new TableRow({ children: [tdCell("Terminal UI", 2800, C.tableRw), tdCode("ratatui + crossterm", 3280, C.tableRw), tdCell("Cross-platform, streaming support", 3280, C.tableRw)] }),
      new TableRow({ children: [tdCell("Vector Store", 2800, C.tableA), tdCode("LanceDB (embedded)", 3280, C.tableA), tdCell("Native Rust, no server required", 3280, C.tableA)] }),
      new TableRow({ children: [tdCell("Relational DB", 2800, C.tableRw), tdCode("SQLite via sqlx", 3280, C.tableRw), tdCell("Session memory, audit log, config", 3280, C.tableRw)] }),
      new TableRow({ children: [tdCell("AST Parsing", 2800, C.tableA), tdCode("tree-sitter", 3280, C.tableA), tdCell("Multi-language, incremental parsing", 3280, C.tableA)] }),
      new TableRow({ children: [tdCell("Plugin System", 2800, C.tableRw), tdCode("mlua + wasmtime", 3280, C.tableRw), tdCell("Lua ringan, WASM untuk plugin advanced", 3280, C.tableRw)] }),
      new TableRow({ children: [tdCell("HTTP Client", 2800, C.tableA), tdCode("reqwest + async", 3280, C.tableA), tdCell("Streaming LLM response support", 3280, C.tableA)] }),
      new TableRow({ children: [tdCell("Serialization", 2800, C.tableRw), tdCode("serde + serde_json", 3280, C.tableRw), tdCell("De-facto standard Rust ecosystem", 3280, C.tableRw)] }),
    ],
  }),
  spacer(1)[0],
  h2("Memory Architecture — 4 Tiers"),
  p("Cantrik menggunakan sistem memori berlapis yang terinspirasi dari cara kerja memori manusia:"),
  new Table({
    width: { size: 9360, type: WidthType.DXA },
    columnWidths: [1800, 2200, 2680, 2680],
    rows: [
      new TableRow({ children: [thCell("Tier", 1800), thCell("Nama", 2200), thCell("Storage", 2680), thCell("Deskripsi", 2680)] }),
      new TableRow({ children: [tdCell("Tier 1", 1800, C.tableRw, false, C.rust), tdCell("Working Memory", 2200, C.tableRw), tdCode("Context Window", 2680, C.tableRw), tdCell("Percakapan aktif sesi ini", 2680, C.tableRw)] }),
      new TableRow({ children: [tdCell("Tier 2", 1800, C.tableA, false, C.rust), tdCell("Session Memory", 2200, C.tableA), tdCode("SQLite per-folder", 2680, C.tableA), tdCell("History sesi, keputusan, context ringkas", 2680, C.tableA)] }),
      new TableRow({ children: [tdCell("Tier 3", 1800, C.tableRw, false, C.rust), tdCell("Project Memory", 2200, C.tableRw), tdCode("LanceDB vector index", 2680, C.tableRw), tdCell("Index semantik seluruh codebase", 2680, C.tableRw)] }),
      new TableRow({ children: [tdCell("Tier 4", 1800, C.tableA, false, C.rust), tdCell("Global Memory", 2200, C.tableA), tdCode("~/.config/cantrik/", 2680, C.tableA), tdCell("Preferensi user, pola kerja, anchors", 2680, C.tableA)] }),
    ],
  }),
  spacer(1)[0],
  h2("Directory Structure"),
  ...codeBlock([
    "~/.config/cantrik/",
    "  config.toml              # Global user preferences",
    "  providers.toml           # API keys & LLM provider config",
    "  anchors.md               # Memory anchors (always in context)",
    "",
    "~/.local/share/cantrik/",
    "  memory.db                # SQLite: session history, audit log",
    "  audit.log                # Human-readable action log",
    "",
    "<project_root>/",
    "  .cantrik/",
    "    cantrik.toml           # Project-level config",
    "    index/                 # LanceDB vector index",
    "    skills/",
    "      backend.md           # Context: arsitektur backend",
    "      database.md          # Context: skema & konvensi DB",
    "      deploy.md            # Context: cara deploy proyek",
    "    sessions/              # Session replay files",
    "    checkpoints/           # Rollback snapshots",
    "    plugins/               # Project-specific Lua plugins",
  ]),
  new Paragraph({ children: [new PageBreak()] }),
];

// ── LLM Bridge ────────────────────────────────────────────────
const llmBridgePage = [
  h1("LLM Bridge — Multi-Provider System"),
  p("Salah satu keunggulan utama Cantrik adalah sistem bridge modular yang memungkinkan plug-and-play provider LLM tanpa mengubah core logic."),
  h2("Provider Matrix"),
  new Table({
    width: { size: 9360, type: WidthType.DXA },
    columnWidths: [2200, 1400, 1400, 1400, 1500, 1460],
    rows: [
      new TableRow({ children: [thCell("Provider", 2200), thCell("Streaming", 1400), thCell("Vision", 1400), thCell("Tool Use", 1400), thCell("Embedding", 1500), thCell("Offline", 1460)] }),
      new TableRow({ children: [tdCell("Anthropic Claude", 2200, C.tableRw), tdCell("✓", 1400, C.tableRw, false, C.rust), tdCell("✓", 1400, C.tableRw, false, C.rust), tdCell("✓", 1400, C.tableRw, false, C.rust), tdCell("—", 1500, C.tableRw), tdCell("—", 1460, C.tableRw)] }),
      new TableRow({ children: [tdCell("Google Gemini", 2200, C.tableA), tdCell("✓", 1400, C.tableA, false, C.rust), tdCell("✓", 1400, C.tableA, false, C.rust), tdCell("✓", 1400, C.tableA, false, C.rust), tdCell("✓", 1500, C.tableA, false, C.rust), tdCell("—", 1460, C.tableA)] }),
      new TableRow({ children: [tdCell("OpenAI / Azure", 2200, C.tableRw), tdCell("✓", 1400, C.tableRw, false, C.rust), tdCell("✓", 1400, C.tableRw, false, C.rust), tdCell("✓", 1400, C.tableRw, false, C.rust), tdCell("✓", 1500, C.tableRw, false, C.rust), tdCell("—", 1460, C.tableRw)] }),
      new TableRow({ children: [tdCell("Ollama (Local)", 2200, C.tableA), tdCell("✓", 1400, C.tableA, false, C.rust), tdCell("✓*", 1400, C.tableA), tdCell("✓*", 1400, C.tableA), tdCell("✓", 1500, C.tableA, false, C.rust), tdCell("✓ FULL", 1460, C.tableA, false, C.rust)] }),
      new TableRow({ children: [tdCell("OpenRouter", 2200, C.tableRw), tdCell("✓", 1400, C.tableRw, false, C.rust), tdCell("✓*", 1400, C.tableRw), tdCell("✓*", 1400, C.tableRw), tdCell("—", 1500, C.tableRw), tdCell("—", 1460, C.tableRw)] }),
      new TableRow({ children: [tdCell("Groq", 2200, C.tableA), tdCell("✓", 1400, C.tableA, false, C.rust), tdCell("—", 1400, C.tableA), tdCell("✓", 1400, C.tableA, false, C.rust), tdCell("—", 1500, C.tableA), tdCell("—", 1460, C.tableA)] }),
    ],
  }),
  p([smoke("* Tergantung model yang digunakan")]),
  spacer(1)[0],
  h2("Smart Routing & Cost Control"),
  p("Cantrik secara otomatis memilih model yang tepat berdasarkan kompleksitas task:"),
  bullet([bold("Task Ringan"), norm(" (rename variable, boilerplate, format): → Model kecil/murah (Haiku, Flash, lokal)")]),
  bullet([bold("Task Sedang"), norm(" (debug, explain code, test writing): → Model menengah (Sonnet, Pro)")]),
  bullet([bold("Task Berat"), norm(" (arsitektur, threading, security audit): → Model besar (Opus, Ultra)")]),
  spacer(1)[0],
  ...codeBlock([
    "# providers.toml",
    "[routing]",
    "auto_route = true",
    "max_cost_per_session = 0.50   # USD",
    "max_cost_per_month   = 10.00  # USD",
    "",
    "[routing.thresholds]",
    "simple  = \"claude-haiku-4\"",
    "medium  = \"claude-sonnet-4\"",
    "complex = \"claude-opus-4\"",
    "",
    "[fallback]",
    "chain = [\"claude-sonnet-4\", \"gemini-flash\", \"ollama/llama3\"]",
  ]),
  spacer(1)[0],
  h2("Embedding Strategy (Offline-First)"),
  p("Indexing codebase dilakukan 100% lokal menggunakan model embedding via Ollama — tidak ada kode yang dikirim ke cloud hanya untuk indexing:"),
  bullet([code("nomic-embed-text"), norm(" — Default, ringan, akurat untuk kode")]),
  bullet([code("mxbai-embed-large"), norm(" — Lebih akurat untuk proyek besar")]),
  bullet([norm("Cloud embedding (OpenAI, Gemini) tersedia sebagai opsi opsional")]),
  new Paragraph({ children: [new PageBreak()] }),
];

// ── Core Features ─────────────────────────────────────────────
const coreFeaturesPage = [
  h1("Fitur Inti"),

  h2("1. Codebase Intelligence (RAG Lokal)"),
  p("Cantrik tidak hanya membaca file secara naif — dia memahami struktur kode secara semantik melalui kombinasi AST parsing dan vector search:"),
  bullet([bold("AST-aware chunking: "), norm("File dipotong berdasarkan boundary fungsi/class, bukan karakter — menghasilkan chunk yang semantically meaningful")]),
  bullet([bold("Dependency graph: "), norm("Cantrik tahu fungsi mana yang memanggil fungsi mana, sehingga bisa menjawab pertanyaan seperti 'Di mana auth_middleware dipakai?'")]),
  bullet([bold("Gitignore-aware: "), norm("Otomatis mengabaikan file di .gitignore, file biner, dan file di atas threshold ukuran")]),
  bullet([bold("Incremental re-index: "), norm("Hanya file yang berubah yang di-index ulang, bukan seluruh codebase")]),
  spacer(1)[0],
  p("Bahasa yang didukung melalui tree-sitter:"),
  ...codeBlock(["Rust, Python, JavaScript, TypeScript, Go, Java, C/C++, PHP, Ruby, SQL, TOML, JSON, YAML, Markdown"]),
  spacer(1)[0],

  h2("2. Multi-Agent Orchestration"),
  p("Cantrik mendukung spawning sub-agent secara paralel untuk task yang dapat di-decompose:"),
  ...codeBlock([
    "Cantrik Orchestrator",
    "├── Sub-agent A: \"Baca semua file auth/\"      → paralel",
    "├── Sub-agent B: \"Cek test coverage module X\" → paralel",
    "└── Sub-agent C: \"Search bug di handler/\"     → paralel",
    "         ↓ semua selesai",
    "   Orchestrator synthesize hasil → jawab user",
  ]),
  spacer(1)[0],
  bullet([bold("Isolated context: "), norm("Setiap sub-agent punya context window terpisah")]),
  bullet([bold("Summary propagation: "), norm("Sub-agent hanya kirim ringkasan ke orchestrator — hemat token")]),
  bullet([bold("Depth limit: "), norm("Sub-agent bisa spawn sub-agent, tapi dengan batas kedalaman (default: 3)")]),
  bullet([bold("Failure isolation: "), norm("Jika satu sub-agent gagal, yang lain tetap jalan")]),
  spacer(1)[0],

  h2("3. Background Agent Mode"),
  p("Cantrik dapat berjalan sebagai daemon di background — kamu bisa tutup terminal dan lanjut kerja:"),
  ...codeBlock([
    "# Jalankan task panjang di background",
    "cantrik background \"refactor semua endpoint ke pattern baru\" --notify",
    "",
    "# Cek status",
    "cantrik status",
    "",
    "# Cantrik pause dan kirim notif jika butuh approval",
    "# Bisa via: desktop notification / webhook / email",
  ]),
  spacer(1)[0],
  bullet([bold("Daemon mode: "), norm("Jalan via systemd user service (Linux) atau launchd (macOS)")]),
  bullet([bold("Checkpoint auto-save: "), norm("Progress tersimpan ke SQLite — tidak hilang jika mati mendadak")]),
  bullet([bold("Notification channels: "), norm("Desktop, webhook URL, atau file flag yang bisa di-poll")]),
  spacer(1)[0],

  h2("4. Long-horizon Planning dengan Re-planning"),
  p("Untuk task kompleks, Cantrik tidak sekadar membuat plan linear — dia mengevaluasi hasil setiap step dan re-plan jika diperlukan:"),
  ...codeBlock([
    "1. Buat initial plan berdasarkan task",
    "2. Eksekusi step 1",
    "3. Evaluasi hasil: apakah sesuai ekspektasi?",
    "   ├── Ya  → lanjut step berikutnya",
    "   └── Tidak → RE-PLAN dengan informasi baru",
    "4. Stuck detection setelah 3 kali gagal → minta bantuan user",
  ]),
  spacer(1)[0],
  p("Cantrik tahu kapan harus menyerah dan meminta bantuan Begawan — ini adalah fitur, bukan kelemahan."),
  spacer(1)[0],

  h2("5. Checkpointing & Rollback"),
  p("Sebelum setiap operasi write, Cantrik otomatis membuat snapshot:"),
  ...codeBlock([
    ".cantrik/checkpoints/",
    "  checkpoint-001-before-auth-refactor/",
    "    src/auth/middleware.rs    # file asli",
    "    src/handlers/login.rs     # file asli",
    "    meta.json                 # timestamp, task description",
    "",
    "# Rollback commands",
    "cantrik rollback              # rollback ke checkpoint terakhir",
    "cantrik rollback --list       # lihat semua checkpoint",
    "cantrik rollback 001          # rollback ke checkpoint spesifik",
  ]),
  new Paragraph({ children: [new PageBreak()] }),

  h2("6. Context Compression Cerdas"),
  p("Saat context window mendekati batas, Cantrik melakukan hierarchical summarization secara otomatis — tanpa user tahu:"),
  bullet([bold("Summarization: "), norm("Percakapan lama diringkas menjadi 500 token, disimpan ke Session Memory")]),
  bullet([bold("Hot context: "), norm("File yang sedang diedit dan error terakhir selalu dipertahankan")]),
  bullet([bold("Memory Anchors: "), norm("Instruksi penting yang TIDAK pernah dihapus dari context:")]),
  spacer(1)[0],
  ...codeBlock([
    "# ~/.config/cantrik/anchors.md",
    "- Selalu gunakan error handling pattern Result<T, E>",
    "- Database schema ada di docs/schema.sql",
    "- Jangan gunakan unwrap() di production code",
    "- Naming convention: snake_case untuk Rust, camelCase untuk JS",
  ]),
  spacer(1)[0],

  h2("7. Sandboxed Execution"),
  p("Cantrik mendukung beberapa level isolasi untuk eksekusi command:"),
  new Table({
    width: { size: 9360, type: WidthType.DXA },
    columnWidths: [2000, 3680, 3680],
    rows: [
      new TableRow({ children: [thCell("Level", 2000), thCell("Implementasi", 3680), thCell("Kapan Dipakai", 3680)] }),
      new TableRow({ children: [tdCode("none", 2000, C.tableRw), tdCell("Raw execution, no isolation", 3680, C.tableRw), tdCell("Developer percaya penuh pada Cantrik", 3680, C.tableRw)] }),
      new TableRow({ children: [tdCode("restricted", 2000, C.tableA), tdCell("bubblewrap (Linux) / sandbox-exec (macOS)", 3680, C.tableA), tdCell("Default — blokir network, batasi fs", 3680, C.tableA)] }),
      new TableRow({ children: [tdCode("container", 2000, C.tableRw), tdCell("Docker container ringan", 3680, C.tableRw), tdCell("Proyek sensitif, code yang tidak dikenal", 3680, C.tableRw)] }),
    ],
  }),
  spacer(1)[0],

  h2("8. Semantic Diff & Merge"),
  p("Cantrik memahami intent dari perubahan, bukan hanya text diff:"),
  ...codeBlock([
    "  Cantrik ingin mengubah fungsi `validate_token`:",
    "",
    "  SEMANTIC CHANGE : Menambahkan expiry check",
    "  AFFECTED        : 3 fungsi yang memanggil validate_token",
    "  RISK            : Low — backward compatible",
    "  TEST COVERAGE   : Belum ada test untuk expiry case",
    "",
    "  Saran: Tambahkan test dulu sebelum apply? [Y/n/e(dit)/v(iew diff)]",
  ]),
  spacer(1)[0],

  h2("9. MCP Integration (Dua Arah)"),
  p("Cantrik beroperasi sebagai MCP server sekaligus bisa consume MCP server lain:"),
  bullet([bold("Sebagai server: "), norm("Claude Desktop, Cursor, atau MCP-compatible tool bisa pakai Cantrik sebagai tool (cantrik serve --mcp)")]),
  bullet([bold("Sebagai client: "), norm("Cantrik bisa memanggil MCP server lain seperti GitHub MCP, Postgres MCP, Browser MCP")]),
  spacer(1)[0],
  h2("10. Provenance & Explainability"),
  p("Setiap baris kode yang ditulis Cantrik memiliki metadata audit:"),
  ...codeBlock([
    "// [cantrik: 2025-07-01T14:23Z | model: claude-sonnet-4 | task: fix auth | conf: high]",
    "if claims.exp < Utc::now().timestamp() {",
    "    return Err(AuthError::TokenExpired);",
    "}",
  ]),

  h2("11. Deep Git-Native Workflow"),
  p("Cantrik dirancang sebagai asisten yang menghormati workflow Git secara native:"),
  bullet([bold("Auto branch creation"), norm(" per task (feature/cantrik-refactor-auth)")]),
  bullet([bold("AI-generated commit message"), norm(" + semantic summary")]),
  bullet([bold("cantrik pr create \"deskripsi\""), norm(" → buat Pull Request ke GitHub/GitLab")]),
  bullet([bold("cantrik fix <github-issue-url>"), norm(" → mode SWE-agent: analisis, fix, test, buat PR")]),
  bullet([bold("Conflict detection"), norm(" + saran resolusi otomatis")]),
  spacer(1)[0],

  h2("12. Structured Plan & Act Mode"),
  p("cantrik plan \"refactor X\" → Cantrik buat rencana terstruktur sebelum eksekusi. Dual-agent: Planner (read-only) dan Builder (dengan approval)."),

  h2("13. Voice-to-Code & TTS"),
  p("Voice input via Whisper lokal + TTS untuk notifikasi (hands-free)."),

  h2("14. Web Research & Sandboxed Browser Tool"),
  p("Tool web_search, browse_page, screenshot dengan approval eksplisit."),

  h2("15. Automated Tech Debt Scanner"),
  p("/health → scan outdated deps, test coverage, security vulns, clippy suggestions."),

  h2("16. Adaptive Begawan Style Learning"),
  p("Cantrik belajar pola coding kamu dari history approval (disimpan di Global Memory)."),

  h2("17. LSP Integration"),
  p("Jalankan sebagai Language Server Protocol untuk integrasi real-time di Neovim / VS Code."),

  h2("18. Visual Codebase Intelligence"),
  p("/visualize → generate Mermaid/PlantUML diagram di TUI."),

  h2("19. Macro & Recipe System"),
  p("Record dan replay workflow kompleks."),

  h2("20. .cantrik/rules.md (Custom Guardrails)"),
  p("File markdown yang selalu di-inject ke context untuk enforce style, security, dan architecture rules."),

  new Paragraph({ children: [new PageBreak()] }),
];

// ── Guardrails ────────────────────────────────────────────────
const guardrailsPage = [
  h1("Guardrails & Sistem Keamanan"),
  p("Filosofi Cantrik sebagai asisten setia tercermin dalam sistem keamanannya. Cantrik tidak pernah bertindak melampaui kehendak Begawan."),

  h2("Permission Tiers"),
  new Table({
    width: { size: 9360, type: WidthType.DXA },
    columnWidths: [1200, 2800, 5360],
    rows: [
      new TableRow({ children: [thCell("Level", 1200), thCell("Status", 2800), thCell("Contoh Operasi", 5360)] }),
      new TableRow({ children: [tdCell("🔴", 1200, C.tableRw), tdCell("FORBIDDEN — hardcoded", 2800, C.tableRw, false, "CC0000"), tdCell("rm -rf sistem, akses file di luar project dir, kirim data ke endpoint asing", 5360, C.tableRw)] }),
      new TableRow({ children: [tdCell("🟡", 1200, C.tableA), tdCell("REQUIRE_APPROVAL — default on", 2800, C.tableA, false, C.rust), tdCell("Write file, eksekusi command, git push/commit, network requests", 5360, C.tableA)] }),
      new TableRow({ children: [tdCell("🟢", 1200, C.tableRw), tdCell("AUTO_APPROVE — default", 2800, C.tableRw, false, "006600"), tdCell("Read file, search codebase, generate suggestion (tanpa apply)", 5360, C.tableRw)] }),
    ],
  }),
  spacer(1)[0],

  h2("Begawan Mode — Autonomy Levels"),
  ...codeBlock([
    "# .cantrik/cantrik.toml",
    "[guardrails]",
    "autonomy_level   = \"supervised\"  # conservative | supervised | autonomous",
    "checkpoint_every = 5             # pause setiap 5 tool call untuk konfirmasi",
    "",
    "require_approval = [\"delete\", \"git push\", \"deploy\", \"curl\", \"wget\"]",
    "auto_approve     = [\"read\", \"search\", \"grep\", \"ls\"]",
  ]),
  spacer(1)[0],

  h2("Audit Log"),
  p("Setiap action yang dieksekusi Cantrik tercatat secara permanen:"),
  ...codeBlock([
    "# ~/.local/share/cantrik/audit.log",
    "[2025-07-01 14:23:11] WRITE  src/auth/middleware.rs  model=claude-sonnet-4  cost=$0.003",
    "[2025-07-01 14:23:45] EXEC   cargo test              approved_by=user",
    "[2025-07-01 14:24:01] READ   src/handlers/login.rs   auto_approved",
    "[2025-07-01 14:24:18] DENIED rm -rf ./              reason=forbidden_pattern",
  ]),
  spacer(1)[0],

  h2("Stuck Detection"),
  p("Cantrik tahu kapan harus berhenti dan meminta bantuan — setelah 3 kali mencoba dengan pendekatan berbeda:"),
  ...codeBlock([
    "  ⚠  Cantrik stuck setelah 3 percobaan.",
    "",
    "  Yang sudah dicoba:",
    "  1. Fix import path           → masih error",
    "  2. Update cargo.toml dep     → masih error",
    "  3. Clean build cache         → masih error",
    "",
    "  Butuh bantuan Begawan. Error terakhir:",
    "  error[E0308]: mismatched types in `auth/middleware.rs:47`",
  ]),
  new Paragraph({ children: [new PageBreak()] }),
];

// ── Terminal UX ───────────────────────────────────────────────
const terminalUXPage = [
  h1("Terminal UX — Ancient Cybernetics CLI"),

  h2("Prompt & Color Scheme"),
  new Table({
    width: { size: 9360, type: WidthType.DXA },
    columnWidths: [2800, 2280, 4280],
    rows: [
      new TableRow({ children: [thCell("Elemen", 2800), thCell("Warna", 2280), thCell("Penggunaan", 4280)] }),
      new TableRow({ children: [tdCode("ꦕꦤ꧀ꦠꦿꦶꦏ꧀ (cantrik) >", 2800, C.tableRw), tdCell("#C9A84C (Gold)", 2280, C.tableRw), tdCell("Prompt identifier, highlight penting", 4280, C.tableRw)] }),
      new TableRow({ children: [tdCell("AI Response text", 2800, C.tableA), tdCell("#E0E0E0 (Base)", 2280, C.tableA), tdCell("Respons utama dari Cantrik", 4280, C.tableA)] }),
      new TableRow({ children: [tdCell("Code blocks", 2800, C.tableRw), tdCell("#C9A84C + #C0540A", 2280, C.tableRw), tdCell("Syntax highlighting, Gold & Rust accent", 4280, C.tableRw)] }),
      new TableRow({ children: [tdCell("Thinking / Logs", 2800, C.tableA), tdCell("#555566 (Smoke)", 2280, C.tableA), tdCell("System logs, proses berpikir, dimmed", 4280, C.tableA)] }),
      new TableRow({ children: [tdCell("Approval prompt", 2800, C.tableRw), tdCell("#DD6B20 (Rust)", 2280, C.tableRw), tdCell("Warning, action yang perlu persetujuan", 4280, C.tableRw)] }),
    ],
  }),
  spacer(1)[0],

  h2("Streaming dengan Visual Thinking"),
  ...codeBlock([
    "ꦕꦤ꧀ꦠꦿꦶꦏ꧀ (cantrik) > fix the authentication bug",
    "",
    "  ◎ Membaca konteks proyek...              [smoke/dimmed]",
    "  ◎ Searching: \"auth\" di AST index...",
    "  ◎ Found: src/auth/middleware.rs, src/handlers/login.rs",
    "  ◎ Reading 2 files (847 tokens)...",
    "",
    "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
    "  Aku menemukan masalahnya. Di middleware.rs baris 47,    [gold]",
    "  token expiry tidak dicek sebelum decode JWT...",
    "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
  ]),
  spacer(1)[0],

  h2("Diff Preview Sebelum Apply"),
  ...codeBlock([
    "  Proposed changes to src/auth/middleware.rs:",
    "",
    "  - let claims = decode_token(token)?;",
    "  + let claims = decode_token(token)?;",
    "  + if claims.exp < Utc::now().timestamp() {",
    "  +     return Err(AuthError::TokenExpired);",
    "  + }",
    "",
    "  SEMANTIC : Menambah expiry validation",
    "  RISK     : Low — backward compatible",
    "",
    "  Apply? [Y/n/e(dit)/d(iff)/s(kip)]",
  ]),
  spacer(1)[0],

  h2("Input Modes"),
  new Table({
    width: { size: 9360, type: WidthType.DXA },
    columnWidths: [3200, 6160],
    rows: [
      new TableRow({ children: [thCell("Command", 3200), thCell("Deskripsi", 6160)] }),
      new TableRow({ children: [tdCode("cantrik", 3200, C.tableRw), tdCell("Interactive REPL mode — percakapan panjang", 6160, C.tableRw)] }),
      new TableRow({ children: [tdCode("cantrik \"do X\"", 3200, C.tableA), tdCell("One-shot mode — satu perintah, langsung selesai", 6160, C.tableA)] }),
      new TableRow({ children: [tdCode("cantrik --plan \"X\"", 3200, C.tableRw), tdCell("Planning mode — buat rencana dulu, user approve, baru eksekusi", 6160, C.tableRw)] }),
      new TableRow({ children: [tdCode("cantrik --watch", 3200, C.tableA), tdCell("Watch mode — monitor file changes, auto-suggest saat error", 6160, C.tableA)] }),
      new TableRow({ children: [tdCode("cargo build 2>&1 | cantrik", 3200, C.tableRw), tdCell("Pipe mode — kirim output command langsung ke Cantrik", 6160, C.tableRw)] }),
      new TableRow({ children: [tdCode("cantrik --from-clipboard", 3200, C.tableA), tdCell("Baca dari clipboard — cocok untuk paste error dari browser", 6160, C.tableA)] }),
      new TableRow({ children: [tdCode("cantrik --image ss.png", 3200, C.tableRw), tdCell("Image input — analisis screenshot UI (model vision)", 6160, C.tableRw)] }),
      new TableRow({ children: [tdCode("cantrik ask \"X\"", 3200, C.tableA), tdCell("Read-only mode — tanya tanpa Cantrik bisa eksekusi apapun", 6160, C.tableA)] }),
    ],
  }),
  spacer(1)[0],

  h2("Built-in Commands"),
  new Table({
    width: { size: 9360, type: WidthType.DXA },
    columnWidths: [2600, 6760],
    rows: [
      new TableRow({ children: [thCell("Command", 2600), thCell("Fungsi", 6760)] }),
      new TableRow({ children: [tdCode("/cost", 2600, C.tableRw), tdCell("Tampilkan usage & biaya session ini + bulan ini", 6760, C.tableRw)] }),
      new TableRow({ children: [tdCode("/memory", 2600, C.tableA), tdCell("Tampilkan state memory semua tier", 6760, C.tableA)] }),
      new TableRow({ children: [tdCode("/index", 2600, C.tableRw), tdCell("Re-index codebase secara manual", 6760, C.tableRw)] }),
      new TableRow({ children: [tdCode("/plan", 2600, C.tableA), tdCell("Minta Cantrik buat rencana sebelum eksekusi", 6760, C.tableA)] }),
      new TableRow({ children: [tdCode("/rollback", 2600, C.tableRw), tdCell("Rollback ke checkpoint terakhir", 6760, C.tableRw)] }),
      new TableRow({ children: [tdCode("/export", 2600, C.tableA), tdCell("Export session ke Markdown", 6760, C.tableA)] }),
      new TableRow({ children: [tdCode("/doctor", 2600, C.tableRw), tdCell("Cek kesehatan semua komponen Cantrik", 6760, C.tableRw)] }),
    ],
  }),

  h2("Enhancement"),
  bullet([bold("TUI Split Pane"), norm(": thinking log | code preview | semantic diff | approval")]),
  bullet([bold("Cultural Wisdom Mode"), norm(" (opsional): sisipkan peribahasa Jawa relevan")]),
  bullet([bold("Multi-root Workspace"), norm(" support")]),

  new Paragraph({ children: [new PageBreak()] }),
];

// ── Plugin System ─────────────────────────────────────────────
const pluginPage = [
  h1("Plugin & Skill System"),
  p("Cantrik menggunakan tiga layer extensibility yang saling melengkapi:"),

  h2("Layer 1: Skill Files (.md)"),
  p("Context tambahan tentang proyek — di-inject otomatis ke context window berdasarkan relevansi task:"),
  ...codeBlock([
    "# .cantrik/skills/backend.md",
    "## Arsitektur Backend",
    "Proyek ini menggunakan Axum sebagai web framework dengan layer:",
    "- Router → Handler → Service → Repository",
    "- Semua error harus menggunakan tipe AppError dari src/errors.rs",
    "- Database: PostgreSQL via sqlx, async only",
  ]),
  spacer(1)[0],

  h2("Layer 2: Lua Plugins (Logic)"),
  p("Plugin dengan logic — untuk workflow otomatis atau custom tools:"),
  ...codeBlock([
    "-- .cantrik/plugins/deploy.lua",
    "function on_task_start(task)",
    "  if task:contains(\"deploy\") then",
    "    cantrik.warn(\"Ingat: pastikan tests passed sebelum deploy!\")",
    "    cantrik.require_approval(\"deploy\")",
    "  end",
    "end",
    "",
    "function after_write(file)",
    "  if file:ends_with(\".rs\") then",
    "    cantrik.suggest(\"cargo clippy -- -D warnings\")",
    "  end",
    "end",
  ]),
  spacer(1)[0],

  h2("Layer 3: WASM Plugins (Advanced)"),
  p("Untuk plugin performa tinggi atau yang ditulis dalam bahasa lain (Go, Python via Wasm):"),
  bullet([norm("Cocok untuk: parser custom, linter khusus, integrator tool berat")]),
  bullet([norm("Sandbox penuh — WASM tidak bisa akses filesystem kecuali diberi izin eksplisit")]),
  bullet([norm("Language-agnostic: compile apapun ke WASM, jalan di Cantrik")]),
  spacer(1)[0],

  h2("Plugin Registry (Community)"),
  ...codeBlock([
    "# Install plugin dari registry",
    "cantrik skill install git-flow",
    "cantrik skill install docker-helper",
    "cantrik skill install laravel-artisan",
    "cantrik skill install prisma-orm",
    "",
    "# List installed",
    "cantrik skill list",
    "",
    "# Update semua",
    "cantrik skill update",
  ]),
  new Paragraph({ children: [new PageBreak()] }),
];

// ── Roadmap ───────────────────────────────────────────────────
const roadmapPage = [
  h1("Roadmap Pengembangan"),

  h2("Phase 0 — Fondasi (Bulan 1-2)"),
  p("Tujuan: Cantrik bisa diinstall dan berfungsi sebagai CLI agent dasar."),
  num([bold("Project setup: "), norm("Cargo workspace, CI/CD GitHub Actions, linting (clippy), formatting (rustfmt)")]),
  num([bold("CLI scaffold: "), norm("clap v4 argument parsing, subcommand structure, shell completion")]),
  num([bold("LLM Bridge v1: "), norm("Anthropic + Gemini + Ollama provider, streaming response")]),
  num([bold("Basic REPL: "), norm("Interactive mode dengan ratatui, colored output dengan crossterm")]),
  num([bold("Config system: "), norm("TOML parsing, global + project config, API key management")]),
  spacer(1)[0],

  h2("Phase 1 — Core Intelligence (Bulan 3-4)"),
  p("Tujuan: Cantrik bisa membaca dan memahami codebase secara semantik."),
  num([bold("tree-sitter integration: "), norm("AST parsing untuk Rust, Python, JS/TS, Go")]),
  num([bold("LanceDB vector store: "), norm("Indexing codebase, semantic search, incremental re-index")]),
  num([bold("Embedding pipeline: "), norm("Ollama nomic-embed-text sebagai default offline embedder")]),
  num([bold("Session Memory: "), norm("SQLite setup, conversation history per-folder, context pruning")]),
  num([bold("File tools: "), norm("read_file, write_file dengan diff preview, approval system")]),
  spacer(1)[0],

  h2("Phase 2 — Agentic Capabilities (Bulan 5-6)"),
  p("Tujuan: Cantrik bisa menjalankan task multi-step secara otonom dengan guardrails."),
  num([bold("Tool system: "), norm("run_command dengan sandboxing, git_ops read-only, web_fetch opsional")]),
  num([bold("Checkpointing: "), norm("Auto-snapshot sebelum write, rollback command")]),
  num([bold("Audit log: "), norm("Setiap action tercatat dengan cost tracking")]),
  num([bold("Stuck detection: "), norm("Re-planning logic, failure threshold, human escalation")]),
  num([bold("Multi-agent v1: "), norm("Orchestrator + sub-agent spawn, parallel execution via tokio")]),
  spacer(1)[0],

  h2("Phase 3 — Advanced Features (Bulan 7-9)"),
  p("Tujuan: Cantrik menjadi tools kelas dunia yang layak dibandingkan Claude Code."),
  num([bold("Background daemon: "), norm("systemd/launchd integration, progress persistence, notifikasi")]),
  num([bold("Plugin system: "), norm("mlua Lua plugins, wasmtime WASM plugins, plugin registry")]),
  num([bold("Smart routing: "), norm("Auto model selection berdasarkan task complexity dan cost budget")]),
  num([bold("MCP integration: "), norm("Cantrik sebagai MCP server + consume MCP server lain")]),
  num([bold("Semantic diff: "), norm("Risk assessment, affected function analysis, test coverage check")]),
  num([bold("Collaborative mode: "), norm("Export/import context, session sharing")]),
  num([bold("Voice-to-Code & TTS")]),
  num([bold("Deep Git-Native Workflow + PR automation")]),
  num([bold("Web Research & Browser Tool")]),
  num([bold("LSP Integration & Visual Intelligence")]),
  num([bold("Macro System & .rules.md")]),
  spacer(1)[0],

  h2("Phase 4 — Ecosystem (Bulan 10-12)"),
  p("Tujuan: Membangun komunitas open source yang aktif di seputar Cantrik."),
  num([bold("cantrik.dev hub: "), norm("Website untuk plugin registry, template sharing, dokumentasi")]),
  num([bold("cantrik init templates: "), norm("Bootstrap project baru dengan template per framework")]),
  num([bold("Air-gapped mode: "), norm("Mode 100% offline untuk enterprise yang tidak boleh kirim kode ke cloud")]),
  num([bold("Package manager integrations: "), norm("Homebrew, apt/deb, pacman, Nix flake, winget")]),
  num([bold("VS Code extension: "), norm("Side panel yang expose Cantrik capabilities ke editor")]),
  num([bold("Desktop companion app (Tauri)")]),
  num([bold("Tech debt scanner & Adaptive Learning")]),
  new Paragraph({ children: [new PageBreak()] }),

  h2("Phase 5 — Maturity & Excellence (baru)"),
  num([bold("Full autonomous SWE-agent mode")]),
  num([bold("Self-improvement pada codebase Cantrik sendiri")]),
  num([bold("Benchmark vs SWE-bench")]),
  new Paragraph({ children: [new PageBreak()] }),
];

// ── Config Reference ──────────────────────────────────────────
const configPage = [
  h1("Referensi Konfigurasi"),

  h2("Global Config (~/.config/cantrik/config.toml)"),
  ...codeBlock([
    "[ui]",
    "theme         = \"ancient-cybernetics\"",
    "show_thinking = true     # Tampilkan proses berpikir Cantrik",
    "stream        = true     # Streaming response",
    "language      = \"id\"     # Bahasa respons Cantrik",
    "",
    "[memory]",
    "vector_model    = \"nomic-embed-text\"",
    "index_strategy  = \"ast_aware\"",
    "compression     = true",
    "anchor_file     = \"~/.config/cantrik/anchors.md\"",
    "",
    "[guardrails]",
    "autonomy_level   = \"supervised\"",
    "checkpoint_every = 5",
    "",
    "[routing]",
    "auto_route           = true",
    "max_cost_per_session = 0.50",
    "max_cost_per_month   = 10.00",
    "",
    "[sandbox]",
    "level = \"restricted\"",
  ]),
  spacer(1)[0],

  h2("Project Config (.cantrik/cantrik.toml)"),
  ...codeBlock([
    "[project]",
    "name    = \"my-api\"",
    "lang    = [\"rust\", \"sql\"]",
    "ignore  = [\"target/\", \".env\", \"*.log\"]",
    "",
    "[memory]",
    "vector_model    = \"nomic-embed-text\"",
    "max_index_size  = \"500MB\"",
    "reindex_on_git_pull = true",
    "",
    "[guardrails]",
    "require_approval = [\"delete\", \"git push\", \"deploy\"]",
    "auto_approve     = [\"read\", \"search\"]",
    "",
    "[skills]",
    "auto_inject = true    # Inject skill files yang relevan otomatis",
    "files = [\"backend.md\", \"database.md\", \"deploy.md\"]",
  ]),
  spacer(1)[0],

  h2("Provider Config (~/.config/cantrik/providers.toml)"),
  ...codeBlock([
    "[providers.anthropic]",
    "api_key = \"${ANTHROPIC_API_KEY}\"",
    "default_model = \"claude-sonnet-4\"",
    "",
    "[providers.gemini]",
    "api_key = \"${GEMINI_API_KEY}\"",
    "default_model = \"gemini-2.5-flash\"",
    "",
    "[providers.ollama]",
    "base_url      = \"http://localhost:11434\"",
    "default_model = \"llama3.3\"",
    "embed_model   = \"nomic-embed-text\"",
    "",
    "[routing]",
    "fallback_chain = [\"anthropic/claude-sonnet-4\", \"gemini/gemini-flash\", \"ollama/llama3.3\"]",
  ]),

  h2("Tambahan di cantrik.toml"),
  ...codeBlock([
    "[ux]",
    "voice_enabled = true",
    "cultural_wisdom = \"light\"     # off | light | full",
    "tui_split_pane = true",
    "",
    "[git]",
    "auto_branch = true",
    "auto_commit = true",
    "pr_provider = \"github\"",
    "",
    "[rules]",
    "custom_rules_file = \".cantrik/rules.md\"",
    "adaptive_learning = true",
  ]),

  new Paragraph({ children: [new PageBreak()] }),
];

// ── Open Source ───────────────────────────────────────────────
const openSourcePage = [
  h1("Open Source & Kontribusi"),

  h2("Lisensi"),
  p([
    bold("Cantrik menggunakan MIT License"),
    norm(" — bebas digunakan, dimodifikasi, dan didistribusikan, termasuk untuk keperluan komersial, selama attribution dipertahankan."),
  ]),
  p("Cantrik tidak mengambil basis dari Claude Code atau tools lain yang bersifat closed-source. Seluruh codebase dibangun dari nol dengan standar etika open source tertinggi."),
  spacer(1)[0],

  h2("Repository Structure"),
  ...codeBlock([
    "cantrik/",
    "├── crates/",
    "│   ├── cantrik-core/      # Core agent logic, context management",
    "│   ├── cantrik-llm/       # LLM bridge & provider implementations",
    "│   ├── cantrik-rag/       # Vector store, AST parsing, indexing",
    "│   ├── cantrik-tools/     # Tool system, file ops, command exec",
    "│   ├── cantrik-tui/       # Terminal UI dengan ratatui",
    "│   ├── cantrik-plugins/   # Lua + WASM plugin runtime",
    "│   └── cantrik-mcp/       # MCP server & client",
    "├── cantrik-cli/           # Binary entry point",
    "├── docs/                  # Dokumentasi",
    "├── plugins/               # Built-in plugins",
    "└── templates/             # cantrik init templates",
  ]),
  spacer(1)[0],

  h2("Cara Berkontribusi"),
  num([bold("Fork & clone"), norm(" repository")]),
  num([bold("Baca"), norm(" CONTRIBUTING.md dan CODE_OF_CONDUCT.md")]),
  num([bold("Pilih issue"), norm(" dengan label good-first-issue atau help-wanted")]),
  num([bold("Buat branch"), norm(" dengan format: feat/nama-fitur atau fix/nama-bug")]),
  num([bold("Submit PR"), norm(" dengan deskripsi jelas dan test yang passing")]),
  spacer(1)[0],

  h2("Standar Kode"),
  bullet([bold("No unsafe Rust"), norm(" kecuali ada justifikasi kuat dan di-review ketat")]),
  bullet([bold("Test coverage"), norm(" minimal 80% untuk semua modul core")]),
  bullet([bold("Dokumentasi"), norm(" untuk semua public API (rustdoc)")]),
  bullet([bold("clippy"), norm(" dan rustfmt wajib passing di CI")]),
  bullet([bold("Conventional Commits"), norm(" untuk semua commit message")]),
  spacer(1)[0],

  h2("Komunitas"),
  bullet([bold("GitHub Discussions"), norm(" — tanya jawab, ide, RFC")]),
  bullet([bold("GitHub Issues"), norm(" — bug report dan feature request")]),
  bullet([bold("cantrik.dev"), norm(" — website resmi, dokumentasi, plugin registry (Phase 4)")]),
  ...spacer(2),
  hr(),
  ...spacer(1),
  new Paragraph({
    alignment: AlignmentType.CENTER,
    children: [
      new TextRun({ text: "ꦕꦤ꧀ꦠꦿꦶꦏ꧀", font: "Noto Serif Javanese", size: 36, color: C.gold }),
      new TextRun({ text: "  ·  Cantrik CLI  ·  Open Source  ·  Built with ", font: "Arial", size: 22, color: C.smoke }),
      new TextRun({ text: "Rust", font: "Consolas", size: 22, color: C.rust }),
    ],
    spacing: { before: 160 },
  }),
];

// ── Assemble document ─────────────────────────────────────────
const doc = new Document({
  numbering: {
    config: [
      { reference: "bullets", levels: [
        { level: 0, format: LevelFormat.BULLET, text: "•", alignment: AlignmentType.LEFT,
          style: { paragraph: { indent: { left: 720, hanging: 360 } } } },
        { level: 1, format: LevelFormat.BULLET, text: "◦", alignment: AlignmentType.LEFT,
          style: { paragraph: { indent: { left: 1080, hanging: 360 } } } },
      ]},
      { reference: "numbers", levels: [
        { level: 0, format: LevelFormat.DECIMAL, text: "%1.", alignment: AlignmentType.LEFT,
          style: { paragraph: { indent: { left: 720, hanging: 360 } } } },
      ]},
    ],
  },
  styles: {
    default: { document: { run: { font: "Arial", size: 22 } } },
    paragraphStyles: [
      { id: "Heading1", name: "Heading 1", basedOn: "Normal", next: "Normal", quickFormat: true,
        run: { size: 36, bold: true, font: "Arial", color: C.dark },
        paragraph: { spacing: { before: 480, after: 200 }, outlineLevel: 0 } },
      { id: "Heading2", name: "Heading 2", basedOn: "Normal", next: "Normal", quickFormat: true,
        run: { size: 28, bold: true, font: "Arial", color: C.accent },
        paragraph: { spacing: { before: 360, after: 160 }, outlineLevel: 1 } },
      { id: "Heading3", name: "Heading 3", basedOn: "Normal", next: "Normal", quickFormat: true,
        run: { size: 24, bold: true, font: "Arial", color: C.rust },
        paragraph: { spacing: { before: 240, after: 120 }, outlineLevel: 2 } },
    ],
  },
  sections: [{
    properties: {
      page: {
        size: { width: 12240, height: 15840 },
        margin: { top: 1440, right: 1440, bottom: 1440, left: 1440 },
      },
    },
    headers: {
      default: new Header({
        children: [new Paragraph({
          children: [
            new TextRun({ text: "ꦕꦤ꧀ꦠꦿꦶꦏ꧀  Cantrik CLI — PRD & Technical Specification", font: "Arial", size: 18, color: C.smoke }),
          ],
          border: { bottom: { style: BorderStyle.SINGLE, size: 4, color: C.gold, space: 4 } },
          spacing: { after: 160 },
        })],
      }),
    },
    footers: {
      default: new Footer({
        children: [new Paragraph({
          children: [
            new TextRun({ text: "Cantrik  ·  Open Source CLI Agent  ·  v1.0", font: "Arial", size: 18, color: C.smoke }),
            new TextRun({ text: "\t\t", font: "Arial" }),
            new TextRun({ text: "Hal. ", font: "Arial", size: 18, color: C.smoke }),
            new SimpleField("PAGE"),
          ],
          tabStops: [
            { type: TabStopType.RIGHT, position: TabStopPosition.MAX },
          ],
          border: { top: { style: BorderStyle.SINGLE, size: 4, color: C.gold, space: 4 } },
          spacing: { before: 160 },
        })],
      }),
    },
    children: [
      ...coverPage,
      ...filosofiPage,
      ...architecturePage,
      ...llmBridgePage,
      ...coreFeaturesPage,
      ...guardrailsPage,
      ...terminalUXPage,
      ...pluginPage,
      ...roadmapPage,
      ...configPage,
      ...openSourcePage,
    ],
  }],
});

Packer.toBuffer(doc).then(buffer => {
  fs.writeFileSync("./cantrik-prd.docx", buffer);
  console.log("Done!");
}).catch(console.error);
