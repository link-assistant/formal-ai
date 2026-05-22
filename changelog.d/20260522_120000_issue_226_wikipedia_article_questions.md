---
bump: patch
---

### Fixed
- Answer Wikipedia article-existence questions such as `есть такая статья в википедии?` with sourced exact or closest-match results instead of falling through to the unknown fallback.
- Avoid treating quoted Russian-language prose as an implicit UI language command.
- Extend the Issue #226 regression to English, Russian, Hindi, and Chinese, with a CI coverage guard that fails if the Wikipedia article-question or UI-language command matrices lose a supported language.
