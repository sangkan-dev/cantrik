# Tree-sitter language extensions (backlog)

## Workspace constraint

`cantrik-core` depends on **tree-sitter 0.26.x**. Grammar crates on crates.io must expose a `LANGUAGE` value compatible with that ABI.

## Adding a language (checklist)

1. Pick a published crate (e.g. `tree-sitter-foo`) and confirm its `Cargo.toml` allows `tree-sitter` **0.26** (not only `>=0.21,<0.23`).
2. Add the dependency to [`crates/cantrik-core/Cargo.toml`](../crates/cantrik-core/Cargo.toml).
3. Extend `Lang` in [`crates/cantrik-core/src/indexing/chunk.rs`](../crates/cantrik-core/src/indexing/chunk.rs): enum variant, `as_str()`, `language()`, `chunk_query()`, `detect_language()` extensions.
4. Add a small unit test that chunks a tiny sample file (follow existing tests in the same module).
5. One language per PR to keep review focused.

## Kotlin note

The popular `tree-sitter-kotlin` crate (0.3.8) currently pins `tree-sitter` to **&lt; 0.23**, so it **cannot** be used in this workspace without a grammar upgrade or fork. Revisit when a 0.26-compatible release exists.

## Bash (sh)

`tree-sitter-bash` (see `crates/cantrik-core/Cargo.toml`) is wired for `.sh` / `.bash` in [`chunk.rs`](../crates/cantrik-core/src/indexing/chunk.rs); follow the same checklist for future grammars.

## CSS

`tree-sitter-css` indexes `.css` files (class/id `rule_set` chunks) in the same module.

## HTML

`tree-sitter-html` indexes `.html` / `.htm` (element / `script_element` / `style_element` with `tag_name`) in the same module.

## Makefile

`tree-sitter-make` indexes `Makefile` / `*.mk` (`rule` targets, `variable_assignment`) in the same module.

## Scala

`tree-sitter-scala` indexes `.scala` / `.sc` (class/object/trait/function definitions with `name`) in the same module.

## References

- Internal indexer entrypoint: [`indexing/mod.rs`](../crates/cantrik-core/src/indexing/mod.rs)
- CONTRIBUTING Phase 5 triage (short pointer)
