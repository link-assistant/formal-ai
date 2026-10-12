---
bump: minor
---

### Added

- A `write_program` cache miss now rediscovers its program from documentation
  instead of answering from a stored template (issue #1165, R1165-1/R1165-2).
  `data/seed/coding-documentation-captures.lino` is pre-cached source data:
  the code blocks the page formalizer reads from byte-for-byte captures of the
  Rust Book, go.dev, kotlinlang.org and the Scala book, each pinned by SHA-256
  and re-derived by `scripts/generate-coding-documentation-captures.mjs`. The
  #1164 decomposer recomposes the page's example with the task's expected
  output bound into its literal, verifies it by decomposing it again and
  against the language's run contract, logs `procedure_cache
  outcome=discovered` with the page and content id, and stores the row in a
  configured runtime cache (`FORMAL_AI_PROCEDURE_CACHE`) so the next request
  is a hit. Both roots: `rediscover_from_documentation` in Rust and
  `rediscoverDocumentedProgram` in the browser worker.

### Changed

- The page formalizer keeps a code block's indentation (R1165-1): only the
  blank lines around a block and the indentation all its lines share are
  dropped, so programs recomposed from captured pages keep their layout.
- The Rust, Go and Kotlin Hello World programs are no longer stored in
  `data/seed/hello-world-programs.lino` or the browser worker's template table
  (R1165-4); they are rediscovered from the documentation captures, and the
  no-memorization ratchet falls from 14 to 11.
