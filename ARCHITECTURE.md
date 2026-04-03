# Architecture Overview

High-level map of the Cantrik repo. For sprint status and ordering, treat **[TASK.md](TASK.md)** as the source of truth; this file summarizes structure and intent.

## Multi-crate workspace

Members: `crates/cantrik-cli` (binary), `crates/cantrik-core` (library). Shared versions live in the root **[Cargo.toml](Cargo.toml)** under `[workspace.dependencies]`.

### `cantrik-core` (library)

Shared logic for CLI and any future clients (HTTP daemon, IDE plugins, etc.).

**Implemented today (`crates/cantrik-core/src/`):**

| Module | Role |
|--------|------|
| `config` | Load and merge TOML: global `~/.config/cantrik/config.toml` + project `.cantrik/cantrik.toml`. Project keys override global. `AppConfig` currently exposes `[ui]` and `[llm]` only. |
| `llm` | `ask_stream_chunks`, provider HTTP clients (Anthropic, Gemini, Ollama, OpenAI-compatible stack), **`llm/providers.rs`** for `providers.toml` (routing, API keys, fallback chain). Not a separate top-level `providers` crate/module — routing lives inside `llm`. |
| `indexing` | `build_index`: tree-sitter (Rust, Python, JS/TS/TSX, Go, Java, C/C++, PHP, Ruby, SQL, TOML, JSON, YAML, Markdown), AST-ish chunks, incremental manifest + reuse, intra-file call edges; artifacts under **`.cantrik/index/ast/`**. |

**Planned (see [TASK.md](TASK.md)):**

| Area | Sprint (per board) |
|------|---------------------|
| Semantic search / LanceDB | 6 |
| Session memory (SQLite, pruning, anchors) | 7 |
| Tool registry, tiers, sandbox | 8+ |

### `cantrik-cli` (binary)

**Layout (`crates/cantrik-cli/src/`):**

- `main.rs` — `#[tokio::main]`; delegates to `cantrik_cli::run().await`.
- `lib.rs` — CLI entry, stdin pipe mode, REPL wiring.
- `cli.rs` — `clap` definitions.
- `repl.rs` — TUI REPL (ratatui / crossterm), streaming via `cantrik_core::llm`.
- `commands/` — `ask`, `plan`, `index`, `doctor`, `completions`.

## Configuration

- **Paths:** `resolve_config_paths` in **[config.rs](crates/cantrik-core/src/config.rs)** — global vs `.cantrik/cantrik.toml`.
- **Merge:** Defaults → global file → project file (project wins per field in `merge`).
- **Supported sections in code today:** `[ui]`, `[llm]` only. Keys such as `[memory]`, `[index]`, theme/streaming in docs/PRD are **not** yet deserialized unless added to `AppConfig`.

`providers.toml` (LLM vendors, keys, `fallback_chain`) is separate: loaded from **`~/.config/cantrik/providers.toml`** by the `llm` layer, not merged into `AppConfig`.

## Async and I/O

- CLI uses **Tokio** for async LLM calls and spawned work (e.g. REPL uses `spawn_blocking` around the TUI loop while streaming uses `tokio::spawn`).
- Not everything is async: config and file reads can be synchronous; avoid claiming “all I/O is async”.

## LLM bridge (actual)

- Entry: **`cantrik_core::llm::ask_stream_chunks`** — builds provider attempt chain from `AppConfig` + `providers.toml`, streams text via callback.
- **No mandatory async trait** for “one provider trait object”; dispatch is concrete modules + shared error type.

## Error handling

- Prefer **`Result`** and **`thiserror`** (e.g. `ConfigError`, `LlmError`). Avoid `unwrap`/`expect` on production paths.
- Example `ConfigError` shape (paths attached to read/parse failures): see **[config.rs](crates/cantrik-core/src/config.rs)**.

## Quality gates

**CI ( [.github/workflows/ci.yml](.github/workflows/ci.yml) ):**

1. `cargo fmt --all -- --check`
2. `cargo check --workspace --all-targets`
3. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
4. `cargo test --workspace --all-targets`

**Optional local hook:** [.githooks/pre-commit](.githooks/pre-commit) runs fmt, clippy, test (without `--workspace`; from repo root this still applies to the workspace).

## Phases vs sprints

PRD phase → sprint mapping is maintained in **[TASK.md](TASK.md)** (“Pemetaan PRD → Sprint”). In short:

- **Phase 0:** Sprints 1–4 (foundation, LLM bridge v1, REPL TUI).
- **Phase 1:** Sprints 5–7 (indexing, vectors, session memory & file tools).
- **Phase 2+:** See TASK board (agentic, advanced, ecosystem).

Do not rely on older “Phase 2 = LLM only” labels; align new prose with that table.

## Design principles

1. **Thin CLI, fat core** — parsing/HTTP/providers favor `cantrik-core`.
2. **Type-safe errors** — enums + `thiserror`, propagate to stderr at CLI boundary.
3. **Pragmatic async** — Tokio where concurrency matters; sync OK for small config/IO.
4. **Incremental delivery** — ship vertical slices per sprint; check TASK.md.

## Extensibility (future)

- Extra treesitter languages, LanceDB, SQLite session store, tool/MCP layers — all sketched in TASK/PRD; crate split (`cantrik-indexing`, etc.) only if compile time or boundaries demand it.

---

See **[TASK.md](TASK.md)** for checklists, **[CONTRIBUTING.md](CONTRIBUTING.md)** for workflow, **[prd/cantrik-doc.js](prd/cantrik-doc.js)** for product requirements.
