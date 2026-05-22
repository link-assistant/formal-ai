---
bump: patch
---

### Fixed
- Issue #232: Answer Russian definition-style Wikipedia disambiguation pages such as `Существо` with their listed meanings instead of falling through to the Wikidata `Animalia` alias.
- Extend the Issue #232 regression to English, Russian, Hindi, and Chinese, with a CI coverage guard that fails if the definition-style disambiguation matrix loses a supported language.
