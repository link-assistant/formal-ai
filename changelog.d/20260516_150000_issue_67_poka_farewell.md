---
bump: patch
---

### Fixed
- **Issue #67 — "пока" not recognised as a valid prompt.** Added a new `farewell` intent to `intent-routing.lino` with keywords for "bye", "goodbye", "пока", "ciao", "再见", "अलविदा" and phrases "до свидания" / "досвидания". Added multilingual farewell responses for English, Russian, Hindi, and Chinese in `multilingual-responses.lino`, farewell examples in `greetings.lino`, and keyword/phrase patterns in `prompt-patterns.lino`. Wired the `Farewell` variant into the Rust engine (`SelectedRule`, `select_rule_for`, `language_aware_answer_for`) and the browser worker (`isFarewellPrompt`, `solve`). The agent now responds with a language-appropriate goodbye instead of the unknown-intent fallback.
