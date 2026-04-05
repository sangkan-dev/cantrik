#!/usr/bin/env bash
# DoD Phase 0 / engineering auto checks (DEFINITION_OF_DONE.md).
# Mirrors .github/workflows/ci.yml plus release build.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# Lance / lance-encoding needs well-known protobuf types (same as CI: protobuf-compiler).
if [[ -z "${PROTOC_INCLUDE:-}" ]]; then
	for d in /usr/include /usr/local/include "${HOME}/.local/protoc-include"; do
		if [[ -f "${d}/google/protobuf/empty.proto" ]]; then
			export PROTOC_INCLUDE="${d}"
			break
		fi
	done
fi

echo "== cargo fmt =="
cargo fmt --all -- --check

echo "== cargo build --release -p cantrik-cli =="
cargo build --release -p cantrik-cli

echo "== cargo clippy =="
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "== cargo test =="
cargo test --workspace --all-targets

echo "== CLI help smoke (via cargo run) =="
cargo run -q -p cantrik-cli -- --help >/dev/null
cargo run -q -p cantrik-cli -- ask --help >/dev/null
cargo run -q -p cantrik-cli -- plan --help >/dev/null
cargo run -q -p cantrik-cli -- index --help >/dev/null
cargo run -q -p cantrik-cli -- doctor --help >/dev/null
cargo run -q -p cantrik-cli -- completions bash >/dev/null

echo "All DoD auto smoke checks passed."
