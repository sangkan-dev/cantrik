#!/usr/bin/env bash
# Called from .github/workflows/benchmark-harness.yml after a shallow SWE-bench clone.
# Keeps the main CI job light; workflow_dispatch only.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
if [[ ! -d .bench/SWE-bench ]]; then
	echo "benchmark-harness-stub: expected .bench/SWE-bench (clone step)" >&2
	exit 1
fi
test -f .bench/SWE-bench/README.md
echo "benchmark-harness-stub: SWE-bench tree present; running non-LLM fix fetch smoke"
cargo test -p cantrik-cli fix_approve_fetch -- --nocapture
echo "benchmark-harness-stub: OK"
