---
bump: patch
---

### Added
- Added the `data/overrides/` grounding override layer beside `data/cache/` with the same per-id structure. Resolution is `(cache or live API) then overrides`: `formal_ai::seed::resolve` decorates a cached external-source record with an override's facts, and every override records why it exists in a `reason` line.
- Added a tree-walking CI suite (`tests/unit/overrides.rs`) that fails when an override references an id with no checked-in cache record, omits its reason, carries no facts, or is redundant (repeats a value the cache already holds), so the layer self-prunes once upstream catches up.

### Changed
- Recorded the issue #398 PR review data-quality standards in `REQUIREMENTS.md` (R278-R283) under the governance rule "latest requirement overrides any earlier one", mapping each CI check to its requirement.
