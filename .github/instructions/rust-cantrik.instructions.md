---
description: "Use when editing Rust code or Cargo files in this repository; enforce Cantrik coding conventions and safe implementation patterns."
applyTo: "src/**/*.rs,crates/**/*.rs,Cargo.toml,**/Cargo.toml"
---

# Rust Implementation Rules

## Architecture
- Keep `main.rs` minimal; move new logic into focused modules.
- Group related code by concern: `cli`, `config`, `llm`, `memory`, `tools`, `agent`.
- Prefer explicit interfaces (traits) for provider-like components.

## Safety and Reliability
- No `unwrap()`/`expect()` in runtime flows.
- Propagate errors with `thiserror`/custom enums when available.
- Validate external input early (CLI args, config values, env vars).

## Async and IO
- Use async only when IO-bound operations justify it.
- Avoid blocking calls inside async context.
- Add timeouts/retries for network operations.

## Testing
- Add unit tests for parser, mapper, and planner logic.
- Add integration tests for CLI command surface where feasible.
- Cover success path and at least one failure path.

## Quality Gate
- Ensure changes are compatible with `cargo fmt`, `cargo clippy`, and `cargo test`.
- Keep diffs focused; avoid unrelated refactors.

## Package
- Install dependencies using `cargo add` to keep `Cargo.toml` tidy.
- Update workspace members if adding new crates.