---
bump: patch
---

### Fixed
- **Issue #50 — "шабат шалом!" not recognised as a greeting.** Added `шалом` as a greeting keyword and `шабат шалом` as a greeting phrase to `intent-routing.lino`, `greetings.lino`, and `prompt-patterns.lino`. The agent now routes these Hebrew-origin greetings (common in Russian-speaking communities) to the `greeting` intent and responds in Russian instead of returning the unknown-intent fallback.
