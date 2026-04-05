# Runsc / micro-VM roadmap (enterprise sandbox)

## Today

- Local: `sandbox.level = "container"` requires `CANTRIK_RUNSC_BIN` (and optional `CANTRIK_RUNSC_RUN_ARGS`); documented in [CONTRIBUTING.md](../CONTRIBUTING.md) § *Enterprise sandbox*.
- CI: GitHub-hosted runners do not provide gVisor; optional workflow [`.github/workflows/runsc-sandbox-smoke.yml`](../.github/workflows/runsc-sandbox-smoke.yml) runs a real `runsc --version` step only when repository variable `CANTRIK_RUNSC_SELF_HOSTED` is `true` and a matching self-hosted runner exists.

## Next (backlog, not MVP)

- **Micro-VM / Firecracker:** separate design for VM lifecycle, image provenance, and Cantrik agent networking; out of scope for default PR CI.
- **PR gate on runsc:** only if the project provides a stable self-hosted pool; keep default `ci.yml` free of privileged installs.

## Acceptance sketch

- Documented operator runbook + one reproducible local command sequence.
- Optional labeled workflow that fails clearly when `runsc` is missing (no silent skip on required checks).
