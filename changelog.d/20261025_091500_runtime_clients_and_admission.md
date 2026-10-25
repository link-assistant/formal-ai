---
bump: patch
---

### Fixed

- Ship pinned Claude, Codex and Playwright MCP in both container variants, initialize the official Claude native linker, and verify local MCP registration and npm availability.
- Check the actual Hive client prerequisites before transferring a task image or creating test issues.
- Keep proposal-only learning events outside release CI gating while retaining mandatory checks and publication verification.
- Report the blocking CI run and bounded, redacted stdout and stderr when a replay command fails.
- Preserve generated-task cleanup refusal details, recognize failed pre-effect admission without phantom tasks, and support a read-only default-token ownership audit.
