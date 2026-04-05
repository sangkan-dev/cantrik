---
name: SWE E2E reliability (high bar)
about: Track product-level autonomous fix workflows before claiming "full SWE" backlog items done
title: "SWE E2E: "
labels: ["swe", "testing"]
---

## Definition

**High reliability** for autonomous SWE-style flows means: reproducible scenario(s), automated regression, and explicit failure modes — not a single manual demo.

## Required for closing "full autonomous SWE-agent"

- [ ] This issue links a **GitHub Actions workflow** (or equivalent) that runs **without** secrets beyond what CI already has, using a **fixture repo** or **recorded HTTP** (no live flaky LLM in the critical path if possible). Baseline hari ini: [`.github/workflows/swe-e2e-smoke.yml`](.github/workflows/swe-e2e-smoke.yml) (wiremock + [`tests/fixtures/cantrik-fix-issue-sample.html`](tests/fixtures/cantrik-fix-issue-sample.html) + unit `fix_cmd::tests`).
- [ ] At least one **end-to-end scenario**: e.g. issue fixture → `cantrik fix …` (or successor) → assert on workspace state / exit codes.
- [ ] **Rollback / safety**: document what happens on failure; no silent destructive defaults.

## Scenario notes

<!-- Commands, fixture URLs, expected artifacts, known limitations. -->
