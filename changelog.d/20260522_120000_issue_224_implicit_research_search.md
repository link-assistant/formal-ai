---
bump: minor
---

### Added
- Issue #224: implicit open-ended research questions such as `What is the most popular dataset for translation quality validation?` now route to the `web_search` pipeline instead of the unknown fallback. The matcher records `web_search:query_kind:implicit_research_question` so diagnostics show why a question without an explicit search verb used external-source gathering.
