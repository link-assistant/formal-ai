---
bump: minor
---

### Added

- A coding procedure is now a deletable, rediscoverable cache entry instead of
  memorized knowledge (issue #1165, E130). The new
  `rust/src/discovery_production.rs` fronts the production miss path —
  `cached_or_research` answers a `(language, task)` pair from
  `data/cache/coding-procedure-cache.lino` and, on a miss, runs
  `research_coding_skill_gap` and stores the verified procedure with its
  rediscovery query and documented source. The policy is data, not code:
  `data/seed/program-cache-policy.lino` declares which fields a row must
  carry, the FNV-1a content address recomputed on every load and store, and
  whether the embedded `ORACLE_SNAPSHOTS` bootstrap in `rust/src/knowledge.rs`
  still answers — retiring it is a seed edit, not a code change.

### Changed

- `CodingOracle::lookup` and `knows_language` serve the embedded snapshots
  only while the policy seed's bootstrap record is active; the snapshots are
  the record of discovery that ran before the cache file existed, and the
  production authority becomes "a grammar exists and discovery found a
  procedure".
