# Benchmark harness (placeholder)

This directory is reserved for **SWE-bench**, **Terminal-Bench**, or other external task slices.

- Local baseline: `./scripts/phase5-smoke.sh` (set `CANTRIK_BENCH_HARNESS=1` when wiring a runner).
- CI: see [`.github/workflows/benchmark-harness.yml`](../.github/workflows/benchmark-harness.yml) (`workflow_dispatch`, does not gate the main Rust job).
- Upstream SWE-bench: clone or submodule [https://github.com/princeton-nlp/SWE-bench](https://github.com/princeton-nlp/SWE-bench) into a subdirectory here in a follow-up iteration.

Do not commit large datasets to this repo without an explicit maintainer decision.
