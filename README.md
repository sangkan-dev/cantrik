# Cantrik

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

- **Website (placeholder docs + plugin registry MVP):** [apps/cantrik-site](apps/cantrik-site/) — deploy to **`https://cantrik.sangkan.dev`**. PRD also mentions `cantrik.dev` as a possible alias/redirect.
- **Pre-built CLI:** Linux x86_64 binary attached to [GitHub Releases](https://github.com/sangkan-dev/cantrik/releases) when you push a version tag `v*` (see `.github/workflows/release.yml`).
- **Packaging:** [Homebrew](packaging/homebrew/cantrik.rb), [nfpm `.deb`](packaging/nfpm.yaml), [Arch `PKGBUILD`](packaging/arch/PKGBUILD), [Nix dev shell](packaging/nix/flake.nix), [Winget manifest](packaging/winget/Sangkan.Cantrik.yaml) (update `InstallerSha256` per release).

### Bootstrap `.cantrik/` in a repo

```bash
cantrik init                    # template generic
cantrik init --template rust-cli
```

---

## Status: Sprint 1 ✓ Foundation & Tooling

**Current Phase:** Foundation Engineering (Multi-crate Rust workspace)

| Component | Status | Details |
|-----------|--------|---------|
| Workspace Structure | ✓ Complete | Multi-crate (core + cli), workspace dependencies |
| Core Dependencies | ✓ Complete | tokio, clap, serde, toml, reqwest, thiserror |
| Config System | ✓ Complete | 2-tier precedence: project > global > defaults |
| Quality Tooling | ✓ Complete | rustfmt, clippy (-D warnings), pre-commit hooks |
| CI/CD | ✓ Complete | GitHub Actions: fmt check, cargo check, clippy, test |
| **Definition of Done** | [/] In Progress | Local: ✓ All green • CI: Awaiting push to trigger |

**Next:** Sprint 2 (CLI Scaffold & Command Surface v1)

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

### Phase 0-1: Foundation (Sprints 1-2)
- [x] **Sprint 1:** Workspace setup, dependencies, config system, tooling
- [ ] **Sprint 2:** CLI scaffold (ask, plan, index, doctor subcommands)

### Phase 2: Core LLM Integration (Sprints 3-4)
- [ ] **Sprint 3:** Multi-provider LLM bridge (Anthropic, Gemini, Ollama)
- [ ] **Sprint 4:** Interactive REPL + thinking log streaming

### Phase 3: Codebase Intelligence (Sprints 5-6)
- [ ] **Sprint 5:** AST indexing via tree-sitter, incremental updates
- [ ] **Sprint 6:** Vector embeddings + semantic search (LanceDB)

### Phase 4: Memory & Context (Sprint 7)
- [ ] **Sprint 7:** Session memory, context compression, memory anchors

**Timeline:** ~4 months (7 x 2-week sprints)

See [TASK.md](./TASK.md) for detailed sprint breakdown and acceptance criteria.

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

**Last Updated:** Sprint 1 Complete (2026-04)  
**Next Milestone:** Sprint 2 — CLI Scaffold & Command Surface v1
