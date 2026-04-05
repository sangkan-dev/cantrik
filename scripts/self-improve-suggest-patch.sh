#!/usr/bin/env bash
# Bounded "suggest patch" pass: writes analysis to .cantrik/ — no git push, no PR, no merge to main.
# Run from repo root after configuring LLM in cantrik.toml. Review output before any follow-up automation.
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"

if ! command -v cantrik >/dev/null 2>&1; then
	echo "self-improve-suggest-patch: install cantrik first (e.g. cargo install --path crates/cantrik-cli)." >&2
	exit 1
fi

mkdir -p .cantrik
OUT=".cantrik/self-improve-suggestions.txt"
STAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
{
	echo "# Cantrik self-improve suggestions (generated $STAMP)"
	echo "# Not a patch; human must review before applying anything."
	echo
} >"$OUT"

cantrik ask "List at most 12 concrete, file-scoped improvement suggestions for this repository (refactors, tests, docs). For each: path or area, one-line rationale, risk level low/med/high. Do not claim to have applied changes." >>"$OUT"

echo "self-improve-suggest-patch: wrote $OUT (review only; no auto-merge)." >&2
