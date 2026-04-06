# MCP for coding agents

- **What MCP is:** separate processes exposing *tools* (filesystem, GitHub, DB, …) that the host (e.g. Cantrik) can call via JSON over stdio.
- **When to suggest MCP:** user needs live data (issues, repos, APIs) beyond static codebase context; prefer MCP over scraping when a server exists.
- **Config:** `providers.toml` → `[[mcp_client.servers]]` with `name`, `command`, `args`. Test with `cantrik mcp call <name> <tool> --json '{}'`.
- **Safety:** each server runs with user’s OS permissions; remind user to review `command`/`args` and env vars (e.g. tokens via env, not committed).
- **Offline / air-gap:** MCP may still reach network depending on server; align with project `[llm].offline` / policy docs.
