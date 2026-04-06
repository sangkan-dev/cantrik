# Writing Cantrik skill packs (.md)

- **Location after install:** files land under `.cantrik/skills/`; the LLM may include them when relevant (see project `[skills]` config).
- **manifest.toml:** `name`, `version`, `files = ["skills/foo.md", ...]` paths relative to package root.
- **Registry path:** `~/.local/share/cantrik/skill-registry/<name>/` with `manifest.toml` + listed files; then `cantrik skill install <name>` from project root.
- **Content style:** short headings, bullet rules, examples; avoid duplicating entire PRD—point to repo paths instead.
- **One concern per file:** e.g. “testing”, “security”, “MCP” so selection stays focused.
