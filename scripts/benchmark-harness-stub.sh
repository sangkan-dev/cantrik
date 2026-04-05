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
mkdir -p .bench
METRICS="$ROOT/.bench/last-stub-metrics.txt"
START_TS="$(date +%s)"
if cargo test -p cantrik-cli fix_approve_fetch -- --nocapture; then
	END_TS="$(date +%s)"
	DUR=$((END_TS - START_TS))
	{
		echo "benchmark_harness_stub_duration_sec=$DUR"
		echo "benchmark_harness_stub_finished_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
		echo "swe_bench_path=.bench/SWE-bench"
	} >"$METRICS"
	echo "benchmark-harness-stub: metrics written to $METRICS"
	echo "benchmark-harness-stub: OK"
else
	END_TS="$(date +%s)"
	DUR=$((END_TS - START_TS))
	{
		echo "benchmark_harness_stub_duration_sec=$DUR"
		echo "benchmark_harness_stub_status=failed"
		echo "benchmark_harness_stub_finished_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
	} >"$METRICS"
	exit 1
fi
