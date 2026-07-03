---
bump: patch
---

### Fixed
- Fixed Codex Responses compatibility by matching shell tool-call arguments to the advertised `cmd` schema, returning `slug` in OpenAI-compatible model metadata, and allowing `with-formal-ai codex` to start outside Git worktrees.
