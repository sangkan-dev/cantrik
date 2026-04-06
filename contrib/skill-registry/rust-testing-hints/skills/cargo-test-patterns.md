# Rust testing patterns

- Prefer **unit tests** in the same file under `#[cfg(test)] mod tests { ... }` for pure logic.
- Use **`tempfile`** or test fixtures for filesystem integration; clean up in `Drop` or explicit teardown.
- For **async**, use `tokio::test` where the crate already depends on tokio with `macros` + `rt`.
- Run **`cargo test -p <crate>`** to scope; **`cargo test --workspace`** before merge when touching shared types.
- **Snapshot / golden tests:** only when output is stable; document update procedure in a comment.
