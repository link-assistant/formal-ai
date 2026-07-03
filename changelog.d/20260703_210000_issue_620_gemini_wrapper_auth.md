---
bump: patch
---

### Fixed

- Isolated one-shot `with-formal-ai gemini` invocations from cached Gemini CLI
  OAuth settings by selecting API-key auth in a temporary Gemini home and
  enabling workspace trust.
