#!/usr/bin/env bash
# Gated automation: Rust tests first; optional LLM dry-run. Never opens PR or merges to main.
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"

echo "self-improve-gated-patch: gate 1 — cargo test --workspace (lib)" >&2
cargo test --workspace --lib

if [[ "${CANTRIK_SELF_IMPROVE_RUN_ASK:-}" == "1" ]]; then
	echo "self-improve-gated-patch: gate 2 — self-improve-dry-run.sh (needs cantrik + provider keys)" >&2
	bash scripts/self-improve-dry-run.sh
else
	echo "self-improve-gated-patch: skip LLM dry-run (set CANTRIK_SELF_IMPROVE_RUN_ASK=1 to enable)" >&2
fi

echo "self-improve-gated-patch: done — no branch push, no PR, no merge to main" >&2
