---
bump: patch
---

### Fixed
- **Issue #445 — compound courtesy/question prompts were treated as one unknown.** The solver now decomposes unresolved independent prompt parts, responds to greetings first, and then answers the following question segment while preserving existing specialized decomposition for algebra and list-style synthesis.
