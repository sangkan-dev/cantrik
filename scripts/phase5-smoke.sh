#!/usr/bin/env bash
# Phase 5 backlog — smoke placeholders (SWE-bench / Terminal-Bench / self-improve harness).
# Extend this script when formal benchmark wiring lands; do not gate releases on it yet.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
echo "== cantrik fmt (workspace) =="
cargo fmt --all -- --check
echo "== cantrik clippy =="
cargo clippy -p cantrik-core -p cantrik-cli -- -D warnings
echo "== cantrik test (workspace lib) =="
cargo test --workspace --lib
echo "phase5-smoke: OK (baseline quality gates only)"
