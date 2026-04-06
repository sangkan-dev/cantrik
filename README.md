# Cantrik

<p align="center">
  <img src="docs/assets/cantrik-logo.png" alt="Cantrik — CLI AI assistant" width="420">
</p>

> **Cantrik** (ꦕꦤ꧀ꦠꦿꦶꦏ꧀) — Open-Source AI CLI Agent berbasis Rust

Cantrik adalah CLI agent yang memahami struktur codebase Anda secara semantik, menggunakan LLM multi-provider dengan kemampuan:

- **Semantic search** berbasis AST dan embeddings lokal
- **Session memory** yang smart dengan context compression
- **Multi-provider LLM** (Anthropic, Google Gemini, Ollama)
- **Interactive REPL** dengan thinking log real-time
- **Codebase indexing** dengan incremental updates

```bash
cantrik ask "apa fungsi dari file ini?"
cantrik plan "refactor database layer"
cantrik index ./src
# Di terminal interaktif, tanpa subcommand: masuk REPL
cantrik
```

### Community & hub

- **Website (hub + dokumentasi pengguna + plugin registry MVP):** [apps/cantrik-site](apps/cantrik-site/) — deploy to **`https://cantrik.sangkan.dev`**. **Dokumentasi pengguna mulai dari `https://cantrik.sangkan.dev/docs`.** PRD also mentions `cantrik.dev` as a possible alias/redirect.
- **Pre-built CLI:** Linux x86_64 binary attached to [GitHub Releases](https://github.com/sangkan-dev/cantrik/releases) when you push a version tag `v*` (see `.github/workflows/release.yml`).
- **Packaging:** [Homebrew](packaging/homebrew/cantrik.rb), [nfpm `.deb`](packaging/nfpm.yaml), [Arch `PKGBUILD`](packaging/arch/PKGBUILD), [Nix dev shell](packaging/nix/flake.nix), [Winget manifest](packaging/winget/Sangkan.Cantrik.yaml) (update `InstallerSha256` per release).

### Bootstrap `.cantrik/` in a repo

```bash
cantrik init                    # template generic
cantrik init --template rust-cli
```

---

## Status & Definition of Done

**Sprint board & roadmap:** lihat [TASK.md](TASK.md).

**Verifikasi objektif vs DoD:** [DEFINITION_OF_DONE.md](DEFINITION_OF_DONE.md) · gate rilis [docs/DOD_RELEASE_GATE.md](docs/DOD_RELEASE_GATE.md) · matriks bukti [docs/DOD_VERIFICATION_MATRIX.md](docs/DOD_VERIFICATION_MATRIX.md) · ringkasan go/no-go [docs/DOD_GO_NO_GO.md](docs/DOD_GO_NO_GO.md).

**Cek otomatis lokal (fmt, `cargo build --release`, clippy, test, help smoke):**

```bash
./scripts/dod-auto-smoke.sh
```

Build LanceDB membutuhkan `protoc` dan _well-known_ protobuf includes (skrip mengisi `PROTOC_INCLUDE` jika ditemukan; di CI Ubuntu paket `protobuf-compiler`).

---

## Quick Start

### Prerequisites

- **Rust 1.70+** (install via [rustup](https://rustup.rs/))
- **Git**

### Development Setup

```bash
# Clone repository
git clone https://github.com/sangkan-dev/cantrik.git
cd cantrik

# Build project
cargo build

# Run CLI (will show config paths and exit)
cargo run --bin cantrik -- --debug-config

# Run tests
cargo test

# Format code
cargo fmt

# Lint (no warnings allowed)
cargo clippy -- -D warnings
```

### Project Structure

```
cantrik/
├── Cargo.toml                      # Workspace root manifest
├── rustfmt.toml                    # Code formatting config
├── .githooks/pre-commit            # Local quality gate
├── .github/
│   ├── copilot-instructions.md     # Copilot project rules
│   ├── instructions/
│   │   ├── rust-cantrik.instructions.md
│   │   └── planning-task.instructions.md
│   ├── skills/
│   │   ├── sprint-task-sync/SKILL.md
│   │   └── rust-cli-feature-delivery/SKILL.md
│   └── workflows/
│       ├── ci.yml                  # Rust: fmt, check, clippy, test
│       ├── cantrik-site.yml       # Svelte hub: check, lint, build
│       └── release.yml            # Release binary on tag v*
├── apps/
│   ├── README.md                  # Hub domain & layout notes
│   └── cantrik-site/              # SvelteKit static hub (Sprint 19)
├── prd/
│   ├── cantrik-prd.md              # Product Requirements Document
│   └── package.json
├── TASK.md                         # Sprint tracking board
└── crates/
    ├── cantrik-core/               # Library: config, providers, tools
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       └── config.rs           # Config loader (global + project)
    └── cantrik-cli/                # Binary: CLI entrypoint
        ├── Cargo.toml
        └── src/
            └── main.rs             # CLI main (tokio async)
```

---

## Architecture Overview

### Multi-Crate Design

**`cantrik-core`** (Library)

- Configuration system (2-tier precedence: `~/.config/cantrik` vs `.cantrik/`)
- LLM provider abstraction (trait-based for future providers)
- Indexing & semantic search engine
- Session memory & context management
- Tool definitions registry

**`cantrik-cli`** (Binary)

- Command-line interface (clap-based)
- Subcommands: `ask`, `plan`, `index`, `doctor`, `health`
- REPL integration
- Config resolution & startup

### Configuration Hierarchy

1. **Project-level** (highest priority): `.cantrik/cantrik.toml`
2. **Global**: `~/.config/cantrik/config.toml`
3. **Defaults** (lowest priority): built-in

Example config:

```toml
[ui]
theme = "dark"

[llm]
provider = "anthropic"
model = "claude-3-sonnet"
```

### Air-gapped / offline LLM (enterprise)

Set `[llm] offline = true` in `cantrik.toml` **or** export `CANTRIK_OFFLINE=1` (also `true` / `yes` / `on`). In that mode Cantrik only uses **Ollama** targets from your provider chain and requires `providers.toml` → `[providers.ollama] base_url` to use a **loopback** host (`127.0.0.1`, `localhost`, or `::1`). Cloud providers in `fallback_chain` are skipped. **`cantrik fetch` / `cantrik web` with `--approve` are refused** in offline mode. MCP, plugins, webhooks, and non-loopback Ollama may still use the network—see **Network surfaces** in `CONTRIBUTING.md`.

### Adaptive Begawan (approval memory, PRD §4.15)

Set `[memory] adaptive_begawan = true` to record summaries when you use `--approve` on `cantrik file write`, `cantrik exec`, and `cantrik experiment`, and to inject a short “recent decisions” block into session LLM prompts. Cap size with optional `[memory] adaptive_begawan_max_chars` (default 900).

### `cantrik health` (optional deeper checks)

Default: configured audit command (e.g. `cargo audit`), `cargo clippy`, `cargo test --workspace --lib`. Optional flags: `--tree` (`cargo tree` depth 2), `--outdated` (`cargo outdated` if the plugin is installed; otherwise reported as skip), `--coverage` (`cargo llvm-cov report --summary-only` if `cargo-llvm-cov` is installed). Use `--soft` for exit 0 on failures.

### Editor and desktop (Sprint 19)

- **VS Code:** [`apps/cantrik-vscode`](apps/cantrik-vscode/) — `npm install && npm run compile`, then “Install from VSIX…” or open folder in VS Code for development; requires `cantrik` on `PATH`. Activity bar **Cantrik** view lists commands and hub/repo links.
- **Companion:** [`apps/cantrik-tray`](apps/cantrik-tray/) — `cargo run` polls `~/.local/share/cantrik/approval-pending.flag` and sends a desktop notification when it appears (same default path as background jobs). **Tauri tray scaffold:** [`apps/cantrik-tauri/README.md`](apps/cantrik-tauri/README.md).

### REPL split pane (Sprint 18)

Set `[ui] tui_split_pane = true` in `cantrik.toml` to show assistant + **preview** columns in the TUI; `/visualize` output appears in the preview when enabled.

---

## Development Workflow

### Pre-Commit Quality Gates

We use automated checks before committing:

```bash
# Installed in .githooks/pre-commit
# Automatically runs:
# 1. cargo fmt --check
# 2. cargo clippy -- -D warnings
# 3. cargo test

# If you see them fail:
cargo fmt                          # Auto-fix formatting
cargo clippy                       # See warnings
cargo test                         # Run tests
```

### Building & Testing

```bash
# Full workflow (as in CI)
cargo fmt                          # Format all code
cargo check --workspace            # Type check
cargo clippy -- -D warnings        # Lint (no errors allowed)
cargo test                         # Unit tests

# Fast iteration
cargo check                        # Quick type check
cargo test lib_name                # Single test
```

### Feature Flags

Currently, `reqwest` is configured with:

- `json` — JSON serialization support
- `rustls` — TLS via rustls (not OpenSSL)

```toml
# In Cargo.toml (workspace deps)
reqwest = { version = "0.13.2", default-features = false,
            features = ["json", "rustls"] }
```

---

## Roadmap

### Phase 0 — Foundation ✅ (Sprints 1–4)

- [x] **Sprint 1:** Workspace, CI, config system
- [x] **Sprint 2:** CLI scaffold (`ask`, `plan`, `index`, `doctor`, completions, one-shot/pipe/REPL)
- [x] **Sprint 3:** Multi-provider LLM bridge (Anthropic, Gemini, Ollama + OpenAI/Azure/OpenRouter/Groq), streaming, fallback
- [x] **Sprint 4:** Interactive REPL + `ratatui` TUI, thinking log, `/cost` `/memory` `/doctor`

### Phase 1 — Core Intelligence ✅ (Sprints 5–7)

- [x] **Sprint 5:** AST indexing via `tree-sitter` (10+ languages), incremental updates, dependency graph
- [x] **Sprint 6:** Vector store (LanceDB), semantic search, Ollama local embeddings
- [x] **Sprint 7:** Session memory (SQLite), context pruning, anchors, `read_file`/`write_file` + diff preview

### Phase 2 — Agentic ✅ (Sprints 8–11)

- [x] **Sprint 8:** Tool registry, sandbox (bubblewrap), permission tiers, `git_ops`
- [x] **Sprint 9:** Checkpoint/rollback, append-only audit log, provenance
- [x] **Sprint 10:** Planning, re-planning, stuck detection & escalation
- [x] **Sprint 11:** Multi-agent orchestration (parallel sub-agents, depth limit, isolation)

### Phase 3 — Advanced Features ✅ (Sprints 12–18)

- [x] **Sprint 12:** Background daemon, `cantrik status`, desktop notifications
- [x] **Sprint 13:** Plugin system (Lua + WASM), skills, rules, macros
- [x] **Sprint 14:** Smart routing, cost budgets, MCP server + client
- [x] **Sprint 15:** Semantic diff, handoff, replay, context export/import
- [x] **Sprint 16:** Git-native workflow, `cantrik review`, web research
- [x] **Sprint 17:** Code archaeology, `cantrik teach`, dependency intelligence, `cantrik audit`
- [x] **Sprint 18:** LSP, voice input, `/visualize` Mermaid, TUI split-pane, cultural wisdom mode

### Phase 4 — Ecosystem ✅ (Sprint 19)

- [x] **Sprint 19:** SvelteKit hub, `cantrik init` templates, GitHub Releases binary, VS Code extension, Tauri tray, health scanner, adaptive Begawan

### Remaining (GA gate)

- [ ] Multi-platform release binaries (Linux aarch64 + macOS — CI workflow updated)
- [ ] Test coverage ≥ 70% (`cargo llvm-cov`)
- [ ] Deploy `apps/cantrik-site` → `cantrik.sangkan.dev`

See [TASK.md](./TASK.md) for the full sprint board and [DEFINITION_OF_DONE.md](./DEFINITION_OF_DONE.md) for release criteria.

---

## Contributing

### Code Standards

- **Language:** Rust 2024 edition
- **Formatting:** `rustfmt` (automatic via CI)
- **Linting:** `clippy` with `-D warnings` (zero warnings policy)
- **Testing:** Unit tests for non-trivial logic
- **Error Handling:** `Result<T, E>` with `thiserror` for custom errors

### Commit Workflow

1. **Create branch** from current sprint:

   ```bash
   git checkout -b sprint-N/feature-name
   ```

2. **Make changes** following [Rust engineering rules](./github/instructions/rust-cantrik.instructions.md)

3. **Pre-commit check** (auto-runs):

   ```bash
   .githooks/pre-commit
   ```

4. **Push and open PR** with:
   - Clear title referencing TASK.md item
   - Link to related issue if applicable
   - CI must pass (GitHub Actions)

### PR Checklist

- [ ] Code passes `cargo fmt --check`
- [ ] Code passes `cargo clippy -- -D warnings`
- [ ] Tests added/updated for new logic
- [ ] TASK.md status updated (`[/]` → `[x]`)
- [ ] CI passes on GitHub

---

## Building for Production

```bash
# Release build (optimized)
cargo build --release

# Binary location
./target/release/cantrik

# Or install locally
cargo install --path crates/cantrik-cli

# Run
cantrik --version
cantrik ask "what does this codebase do?"
```

---

## Troubleshooting

### Build Errors

**"cannot find type `Config` in module `config`"**

- Ensure `src/config.rs` exports public types
- Check: `pub struct AppConfig { ... }`

**"feature `rustls-tls` is not valid"**

- Use `rustls` feature (not `rustls-tls`)
- See [Cargo.toml](./Cargo.toml) for correct configuration

### Test Failures

```bash
# Run with output
cargo test -- --nocapture

# Single test
cargo test config_overrides -- --exact
```

---

## License

MIT License — See LICENSE file for details

---

## Contact & Support

- **Issues:** [GitHub Issues](https://github.com/sangkan/cantrik/issues)
- **PRD & Design:** [prd/cantrik-prd.md](./prd/cantrik-prd.md)
- **Sprint Board:** [TASK.md](./TASK.md)

---

## Acknowledgments

Cantrik is built with:

- [Rust Language](https://www.rust-lang.org/)
- [Tokio](https://tokio.rs/) — Async runtime
- [Clap](https://docs.rs/clap/) — CLI argument parsing
- [Serde](https://serde.rs/) — Serialization
- [Anthropic Claude](https://claude.ai/) / [Google Gemini](https://gemini.google.com/) / [Ollama](https://ollama.com/) — LLM providers

---

**Last Updated:** Sprint 1–19 Complete (2026-04)  
**Next Milestone:** GA — multi-platform binaries, coverage ≥70%, docs site deploy
