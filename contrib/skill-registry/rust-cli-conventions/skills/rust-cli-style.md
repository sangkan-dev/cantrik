# Rust CLI conventions (Cantrik-style)

When editing or reviewing Rust CLI code in this repo style:

- Prefer `Result<T, E>` on error paths; avoid `unwrap()`/`expect()` in production paths.
- Use `clap` for argument parsing; subcommands map to clear user stories.
- Keep modules focused: split parser, config, IO when files grow.
- Run `cargo fmt`, `cargo clippy -D warnings`, and `cargo test` before suggesting “done”.
- For config: TOML under `.cantrik/` or XDG paths; document env overrides in help text.
