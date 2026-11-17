---
bump: patch
---

### Fixed

- Apply system and developer output constraints within the current Chat Completions or Responses request, including language, JSON format, allowed labels and word counts. Unsupported constraints are named explicitly in a refusal and recorded in `thinking_steps` (#1587).
- Recall facts stated in earlier user turns without treating assistant text as user evidence or sharing request instructions across conversations.
