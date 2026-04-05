#!/usr/bin/env bash
# Constrained SWE-style demo: one public issue URL + local repo (Phase 5 backlog).
# Requires: cantrik on PATH, LLM configured, network for fetch unless cached.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
: "${ISSUE_URL:?set ISSUE_URL to an https issue URL, e.g. https://github.com/org/repo/issues/1}"
echo "ISSUE_URL=$ISSUE_URL"
echo "Running: cantrik fix (fetch + optional agents) — review HTML on stdout before trusting agents output."
cantrik fix "$ISSUE_URL" --approve --fetch --run-agents
echo "swe-fix-demo: done."
