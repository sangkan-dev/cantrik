# Cantrik Workspace Overview

Cantrik adalah open-source AI CLI agent untuk memahami dan berinteraksi dengan codebase Anda menggunakan multi-provider LLM dengan kemampuan semantic search, session memory, dan agentic execution.

**Repository:** [sangkan-dev/cantrik](https://github.com/sangkan-dev/cantrik)  
**Documentation:** [README.md](README.md) • [PRD](prd/cantrik-doc.js) • [TASK.md](TASK.md)  
**Contributing:** See [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Quick Links

| Resource | Link |
|----------|------|
| **README & Setup** | [README.md](README.md) |
| **Sprint Board** | [TASK.md](TASK.md) |
| **Contributing Guide** | [CONTRIBUTING.md](CONTRIBUTING.md) |
| **PRD Document** | [prd/cantrik-doc.js](prd/cantrik-doc.js) |
| **Copilot Rules** | [.github/copilot-instructions.md](.github/copilot-instructions.md) |
| **GitHub Workflows** | [.github/workflows/ci.yml](.github/workflows/ci.yml) |

---

## Current Status

**Phase:** Foundation & Tooling (Sprint 1 Complete ✓)

- ✅ Workspace setup (multi-crate)
- ✅ Dependencies configured
- ✅ Config system scaffolded
- ✅ CI/CD pipeline
- 🚧 CLI scaffold (Sprint 2)

See [TASK.md](TASK.md) for full sprint breakdown.

---

## Getting Started

```bash
# Development setup
git clone https://github.com/sangkan-dev/cantrik.git
cd cantrik
cargo build
cargo test

# Run CLI
cargo run --bin cantrik -- --debug-config
```

See [README.md](README.md#quick-start) for detailed setup.

---

## Code Standards

- **Language:** Rust 2024 edition
- **Format:** `cargo fmt` (no manual formatting)
- **Lint:** `cargo clippy -- -D warnings` (zero warnings)
- **Tests:** Unit tests for logic
- **Errors:** `Result<T, E>` with `thiserror`

Run locally via `.githooks/pre-commit` or manually:
```bash
cargo fmt && cargo check && cargo clippy -- -D warnings && cargo test
```

---

## Contributing

1. Read [CONTRIBUTING.md](CONTRIBUTING.md)
2. Pick or create issue
3. Create feature branch: `git checkout -b sprint-N/feature-name`
4. Code + test + commit (conventional commits)
5. Push + open PR with checklist
6. Address review feedback
7. Merge when approved

---

## Architecture

**Multi-crate design:**
- `cantrik-core` — Library (config, LLM, indexing, memory)
- `cantrik-cli` — Binary (CLI, commands, REPL)

**Config system:**
- Project-level: `.cantrik/cantrik.toml` (highest priority)
- Global: `~/.config/cantrik/config.toml`
- Defaults: Built-in (lowest priority)

See [README.md#architecture-overview](README.md#architecture-overview) for details.

---

## Roadmap

| Sprint | Phase | Focus |
|--------|-------|-------|
| 1 | Foundation | Workspace, deps, config, tooling ✅ |
| 2 | CLI Scaffold | Subcommands (ask, plan, index, doctor) |
| 3 | LLM Bridge | Multi-provider abstraction |
| 4 | TUI/REPL | Interactive mode |
| 5 | Indexing | AST + semantic understanding |
| 6 | Search | Vector embeddings + semantic query |
| 7 | Memory | Session persistence + compression |
| 8-10 | Advanced | Tools, audit, agentic execution |

**Timeline:** ~4 months (7-10 x 2-week sprints)

Full details: [TASK.md](TASK.md)

---

## License

MIT License — See [LICENSE](LICENSE)

---

## Support

- 📝 [Open an Issue](https://github.com/sangkan-dev/cantrik/issues)
- 💬 [GitHub Discussions](https://github.com/sangkan-dev/cantrik/discussions) (if enabled)
- 📖 [README](README.md) + [CONTRIBUTING](CONTRIBUTING.md)

---

**Last Updated:** Sprint 1 Complete (April 2026)
