# Architecture Overview

This document describes the high-level design of Cantrik.

## Multi-Crate Structure

### `cantrik-core` (Library)

Core logic shared by all client implementations (CLI, future: API server, IDE plugins).

**Modules:**
- **config** — Configuration loading (2-tier precedence: project > global > defaults)
- **providers** (Sprint 3) — LLM provider abstraction (trait-based)
  - Anthropic (Claude)
  - Google Gemini
  - Ollama (local)
- **indexing** (Sprint 5) — Codebase semantic understanding
  - AST parsing via tree-sitter
  - File scanning (.gitignore-aware)
  - Incremental indexing
- **search** (Sprint 6) — Semantic search engine
  - Vector embeddings (LanceDB)
  - Metadata indexing
  - Query API
- **memory** (Sprint 7) — Session persistence
  - SQLite backend
  - Memory compression
  - Context pruning
- **tools** (Sprint 8) — Tool system
  - Tool registry
  - Permission tiers
  - Audit logging

### `cantrik-cli` (Binary)

CLI entrypoint and user-facing commands.

**Structure:**
- `main.rs` — Tokio async runtime, clap dispatch
- `commands/` (future) — Individual subcommand implementations
  - `ask.rs` — Query codebase with LLM
  - `plan.rs` — Planning/refactoring assistance
  - `index.rs` — Manage codebase index
  - `doctor.rs` — Diagnostics
- `repl.rs` (Sprint 4) — Interactive REPL mode

## Configuration System

### Precedence

1. **Project-level** (highest): `.cantrik/cantrik.toml`
2. **Global**: `~/.config/cantrik/config.toml`
3. **Defaults** (lowest): Built-in structure

### Format

TOML with sections:

```toml
[llm]
provider = "anthropic"
model = "claude-3-sonnet"
# ... provider-specific settings

[ui]
theme = "dark"
streaming = true

[memory]
# ... session memory settings

[index]
# ... indexing configuration
```

### Properties

- Partial configs valid (missing sections use defaults)
- Project config overrides specific keys from global
- Environment variables can override: `CANTRIK_LLM_PROVIDER=gemini`

## Async Runtime

**Runtime:** Tokio 1.50+ (multi-threaded)
- CLI: single-threaded tokio works fine
- Future: could scale to multi-request scenarios

**Patterns:**
- All I/O operations return `Future`
- Use `#[tokio::main]` in main.rs
- Providers implement async trait methods

## Error Handling

**Strategy:** `Result<T, E>` with `thiserror`

```rust
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    Read(#[from] std::io::Error),
    
    #[error("invalid TOML: {0}")]
    Parse(#[from] toml::de::Error),
}
```

**Policy:** No `unwrap()` in production code paths. Propagate errors or handle gracefully.

## Dependency Management

**Workspace dependencies** (`Cargo.toml`):
```toml
[workspace.dependencies]
clap = { version = "4.6.0", features = ["derive"] }
tokio = { version = "1.50.0", features = ["macros", "rt-multi-thread"] }
# ... etc
```

Members inherit via `.workspace = true`:
```toml
[dependencies]
clap.workspace = true
```

**Rationale:** Centralized version control, easier upgrades, consistent versions across crates.

## Quality & Testing

### Local Gates

- **Format check:** `cargo fmt --check`
- **Type check:** `cargo check --workspace`
- **Lint:** `cargo clippy -- -D warnings` (no warnings allowed)
- **Tests:** `cargo test`

Automated via `.githooks/pre-commit`

### Unit Tests

- Place tests in same file with `#[cfg(test)] mod tests`
- Test non-trivial logic
- Use descriptive names: `fn test_config_project_overrides_global()`

### CI Pipeline

GitHub Actions (`.github/workflows/ci.yml`):
1. Checkout
2. Install Rust (dtolnay/rust-toolchain@stable)
3. Cache dependencies
4. Parallel jobs:
   - `cargo fmt --check`
   - `cargo check --workspace`
   - `cargo clippy -- -D warnings`
   - `cargo test --lib`

## Phases & Milestones

### Phase 0-1: Foundation (Sprints 1-2) ✅ In Progress
- Multi-crate workspace ✅
- Config system ✅
- CLI scaffold (in progress)

### Phase 2: LLM Integration (Sprints 3-4)
- Multi-provider abstraction
- REPL + streaming

### Phase 3: Codebase Intelligence (Sprints 5-6)
- AST indexing
- Semantic search

### Phase 4: Memory & Safety (Sprints 7-9)
- Session persistence
- Tool system + guardrails
- Audit logging + rollback

### Phase 5: Agentic Execution (Sprints 10+)
- Re-planning loops
- Complex task decomposition

## Design Principles

1. **Modular:** Features in separate crates when scale grows
2. **Type-safe:** Leverage Rust's type system to prevent runtime errors
3. **Async-first:** All I/O uses async/await
4. **Testable:** Unit tests for logic, integration tests for workflows
5. **Documented:** Doc comments + architecture docs
6. **Incremental:** Ship working features early, iterate

## Future Extensibility

**Potential extensions:**
- **Web API:** REST server wrapping cantrik-core
- **IDE Plugins:** VS Code, Sublime Text using core library
- **Language Support:** Additional tree-sitter languages
- **Providers:** Local LLMs via additional backends
- **Persistence:** Distributed session storage

Architecture supports these without major rewrites.

---

See [TASK.md](../TASK.md) and [CONTRIBUTING.md](../CONTRIBUTING.md) for sprint details and contribution workflow.
