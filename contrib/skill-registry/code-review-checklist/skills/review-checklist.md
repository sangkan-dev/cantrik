# Code review checklist (agent)

Use when reviewing a PR or diff:

1. **Correctness** — Does the change match the stated goal? Edge cases?
2. **Tests** — New logic covered? Existing tests still pass?
3. **Security** — User input sanitized? No secrets in logs or repo?
4. **Performance** — Hot paths acceptable? Unbounded allocations or I/O?
5. **API / UX** — CLI flags and errors clear for humans?
6. **Docs** — README, `--help`, or comments updated if behavior changed?

Keep feedback specific: file, suggestion, optional patch idea.
