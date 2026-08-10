---
bump: minor
---

### Added
- Procedural "how to X" requests now synthesise one ordered guide from the enabled trusted services in `data/seed/sources-registry.lino`, recursively capturing result pages within declared depth, page, and age bounds and keeping the exact source URL, license, and payload digest on every accepted step.
- Per-service accessibility (success *and* failure) is remembered in the environment's associative memory for seven days, with explicit refresh and invalidation, so a stale body cache is no longer mistaken for an availability record.
- Committed real-service QA captures with timestamps, digests, and licenses; the normal test suite replays them offline on the native, HTTP, and browser paths, and a `FORMAL_AI_LIVE_FETCH=1` refresh check detects drift against the live services.
- The reader-facing guide is rendered from seeded prose (`data/seed/multilingual-responses-procedure.lino`), so `HowToGuide::markdown_in` and the browser worker render the same evidence in any seeded language, while trace and evidence lines are `key=value` records built through the new `trace_record` module.
