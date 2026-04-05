---
name: SWE E2E reliability (high bar)
about: Track product-level autonomous fix workflows before claiming "full SWE" backlog items done
title: "SWE E2E: "
labels: ["swe", "testing"]
---

## Definition

**High reliability** for autonomous SWE-style flows means: reproducible scenario(s), automated regression, and explicit failure modes — not a single manual demo.

## Required for closing "full autonomous SWE-agent"

- [ ] This issue links a **GitHub Actions workflow** (or equivalent) that runs **without** secrets beyond what CI already has, using a **fixture repo** or **recorded HTTP** (no live flaky LLM in the critical path if possible). Baseline hari ini: [`.github/workflows/swe-e2e-smoke.yml`](.github/workflows/swe-e2e-smoke.yml) — `cargo test … commands::fix_cmd::fetch_integration::` (wiremock, redirect, mini workspace, fixture HTML) + `fix_cmd::tests` + validasi [`tests/fixtures/catalog.json`](tests/fixtures/catalog.json).
- [ ] At least one **end-to-end scenario** beyond redirect + mini-workspace: mis. multi-file assert, atau checkout fixture repo terpisah → `cantrik fix …` → assert artefak di disk (bukan hanya exit code).
- [ ] **Rollback / safety**: document what happens on failure; no silent destructive defaults.

## Scenario notes

<!-- Commands, fixture URLs, expected artifacts, known limitations. -->
