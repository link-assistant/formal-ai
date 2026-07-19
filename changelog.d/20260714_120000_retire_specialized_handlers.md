---
bump: patch
---

### Changed
- Retired the `SPECIALIZED_HANDLERS` precedence remnant into data-driven routing: the dispatch ordering now lives in `data/seed/handler-precedence.lino` and is joined with the Rust function pointers at dispatch-build time, with a permutation assertion guarding against silent handler drops or duplicates.

### Added
- Handler-precedence auto-learning report: Formal AI re-derives the specialized-handler precedence itself through its own Agent CLI, ranking the persisted precedence rationale (`data/meta/issue-663-handler-precedence-learning.lino`) into a human-review-gated proposal whose committed evidence is byte-for-byte reproducible by the in-process renderer.

### Fixed
- General-change write routing no longer claims a request whose recovered payload is only a non-referential subject (a bare pronoun such as "it"/"this"): "save it to FILE" pointed back at content a keyword recipe must still compose, so the generic write probe now declines and lets that recipe author the real artifact instead of writing the literal word "it".
