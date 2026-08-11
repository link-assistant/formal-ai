---
bump: patch
---

### Fixed

- Local location, conversation-preference, correction, associative-memory,
  British `behaviour`, and unquoted teaching prompts now stay on their seeded
  symbolic routes instead of falling through to unrelated web/document plans;
  failed web transports retain their real diagnostic, and asking what Links
  Notation is no longer starts document generation (#989).
- Agentic English narration no longer repeats the subjective word `quick` after
  a user rejects it (#989).
- GitHub issue reports can attach harness, server, and merged context as three
  separate links, with safe filenames and valid link-only Markdown (#989).
