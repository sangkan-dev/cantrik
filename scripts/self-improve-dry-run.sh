#!/usr/bin/env bash
# Read-only dry-run: run a short `cantrik ask` on this repo before any self-serve automation.
# Tune cost/context in ~/.config/cantrik/cantrik.toml or project `.cantrik/cantrik.toml`.
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"

if ! command -v cantrik >/dev/null 2>&1; then
  echo "self-improve-dry-run: install cantrik first (e.g. cargo install --path crates/cantrik-cli)." >&2
  exit 1
fi

echo "self-improve-dry-run: one bounded read-only question (no --approve, no writes)." >&2
# Optional: export CANTRIK_SELF_IMPROVE_MAX_TOKENS=… if you add support in config; documented as intent only.
cantrik ask "In at most 8 bullet points, name concrete risks or tech-debt hotspots in this repository (no file changes; analysis only)."
