---
bump: minor
---

### Fixed
- A bare numeric-list follow-up no longer answers `unknown` (issue #412). After
  a turn establishes a coding context — e.g. "…отсортируй их в JavaScript, дай
  мне код и результат" — a follow-up that names no language and does not ask for
  code, such as `Отсортируй 4, 3, 1, 17, 8, 9, 15`, now recovers the target
  language (and the code request) from the conversation and continues the coding
  context: idiomatic code in the established language plus the deterministically
  computed result.

### Added
- Conversational coreference for the numeric-list coding path. A new
  `numeric_list_history_context` inherits the language / code request from a
  prior turn **only** when that turn was itself a genuine numeric-list coding
  request (a recognised operation, a supported program language, and ≥2 numbers),
  so unrelated chatter never leaks a language. A `numeric_list_coreference`
  trace event records what was inherited. Implemented identically in the Rust
  solver (`src/solver_handlers/numeric_list/mod.rs`) and the browser worker
  mirror (`src/web/formal_ai_worker.js`); the 170-cell cross-runtime parity
  matrix stays byte-identical.
