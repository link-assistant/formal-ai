---
bump: patch
---

### Changed
- Retired the `SPECIALIZED_HANDLERS` precedence remnant into data-driven routing: the dispatch ordering now lives in `data/seed/handler-precedence.lino` and is joined with the Rust function pointers at dispatch-build time, with a permutation assertion guarding against silent handler drops or duplicates.
