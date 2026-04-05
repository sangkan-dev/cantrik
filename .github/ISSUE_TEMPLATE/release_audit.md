---
name: Release / air-gap audit
about: Maintainer checklist before tagging or deploying to a restricted environment
title: "Release audit: "
labels: ["release", "ops"]
---

Use [CONTRIBUTING.md — Enterprise release checklist](https://github.com/sangkan-dev/cantrik/blob/main/CONTRIBUTING.md#enterprise-release-checklist) and tick items below.

## Checklist

- [ ] MCP servers in `providers.toml`: spawn surface + outbound network reviewed
- [ ] Plugins (Lua/WASM): trusted source + tool exposure
- [ ] `[background].webhook_url`: destination, TLS, data handling
- [ ] Ollama / embeddings: loopback vs remote; data residency
- [ ] `CANTRIK_OFFLINE` / `[llm].offline`: matches policy for fetch, tools, webhooks
- [ ] Packaging: PKGBUILD `sha256sums` and winget `InstallerSha256` updated for this tag (if applicable)

## Notes

<!-- Links to PRs, signing, SBOM, customer-specific overrides, etc. -->
