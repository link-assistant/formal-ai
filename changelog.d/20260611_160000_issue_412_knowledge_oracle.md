---
bump: minor
---

### Added
- Coding oracle backed by external knowledge sources (issue #412, R6). The
  solver now treats Rosetta Code, Wikifunctions, the Hello World Collection, and
  Stack Overflow as cached external APIs (even though they expose no machine
  API) and generalises `write_program` beyond the verified catalogue: a request
  for a language the catalogue does not template — Kotlin, Swift, PHP, Bash, Lua,
  Haskell — now returns a reviewed snippet, its deterministic output, and its
  source attribution instead of dead-ending on the unsupported answer.
  Catalogued languages keep their verified "compiled and ran" route untouched;
  the oracle only ever supplies an answer the solver would otherwise lack. New
  module `src/knowledge.rs` (sources + `CodingOracle`) and handler
  `src/solver_handler_oracle.rs`, mirrored byte-for-byte in the browser worker
  (`src/web/formal_ai_worker.js`).
- Bounded-cache policy (issue #412, R8). `cache_capacity` /
  `within_cache_capacity` / `KNOWLEDGE_CACHE_FLOOR` enforce "never cache more
  than 1% of a source, or 512 items when 1% is smaller", clamped to the source
  size, for every per-source / per-topic cache. A ratchet test fails CI if the
  committed snapshot set ever exceeds the cap, so a cache can never silently grow
  into a mirror.
