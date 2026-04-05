# Nix flake (`flake.nix`)

## What this provides

- **`devShells.default`** — Rust toolchain, `clippy`, `rustfmt`, `protobuf`, `openssl` for developing Cantrik on Linux/macOS.

## Usage

From the **repository root**:

```bash
nix develop ./packaging/nix
# then, from repo root:
cargo build --release -p cantrik-cli
```

## Packaged `cantrik` derivation

A full `nix build` of the workspace (with LanceDB, WASM, etc.) is **not** provided here yet: dependency closure is large and belongs in a dedicated flake iteration or nixpkgs proper. Until then, use the dev shell + `cargo install --path crates/cantrik-cli` as documented in [`../README.md`](../README.md).
