# Fix / fetch test fixtures (catalog)

Stable bodies for non-LLM regression tests. CI uses wiremock + these files; do not hot-link flaky third-party HTML.

Machine-readable index: [`catalog.json`](catalog.json) (extend when adding files; optional CI validation TBD).

| ID | File | Used by |
|----|------|---------|
| `cantrik-fix-issue-sample` | [`cantrik-fix-issue-sample.html`](cantrik-fix-issue-sample.html) | `fix_cmd::fetch_integration::fix_approve_fetch_reaches_fixture_file` ([`crates/cantrik-cli/src/commands/fix_cmd.rs`](../../crates/cantrik-cli/src/commands/fix_cmd.rs)) |

**Optional live URL (maintainer / fork only):** set env `CANTRIK_FIX_E2E_HTTP_URL` to a raw URL serving the same bytes (or equivalent HTML). Repository variable `CANTRIK_FIX_CI_PINNED_URL` triggers an optional job in [`.github/workflows/swe-e2e-smoke.yml`](../../.github/workflows/swe-e2e-smoke.yml) when non-empty.

**Adding a fixture:** add the file here, reference it with `include_str!` from a `#[cfg(test)]` module, extend this table, and add a wiremock test (no LLM in assertions).
