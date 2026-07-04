---
bump: patch
---

### Fixed

- Agent-mode OpenAI-compatible tool planning now routes local file-reading
  prompts to `read`/`bash` tool calls instead of treating filenames such as
  `beta.md` as URLs or falling through to non-agentic answers.
