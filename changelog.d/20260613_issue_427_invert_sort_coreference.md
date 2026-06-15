---
bump: minor
---

### Fixed
- **Bare "invert the sort" follow-up no longer answers `unknown` (issue #427).**
  After a numeric-list sorting conversation, the bare follow-up
  "Сделай инверсию сортировки." (make the inversion of the sort) fell through to
  `unknown`: the operation vocabulary did not recognize the *invert* phrasing as
  a descending sort, and even when an operation was named the handler had no
  numbers to act on because the follow-up lists none of its own. The
  numeric-list handler now inherits the list from the most recent operation turn
  that carried a concrete list — while the language and code request keep coming
  from the most recent turn that named a language (issue #412) — so a
  number-less invert-sort continues the established coding context and emits the
  descending code plus result.

### Added
- `reverse_sort` operation vocabulary now matches *invert*-style phrasings across
  every supported language: English `invert the sort` / `invert the sorting` /
  `invert sort` and `combo invert+sort`; Russian `combo инверс+сортиров` /
  `combo инверт+сортиров`; Hindi `combo उलट+क्रम`; Chinese `combo 反转+排序` /
  `combo 颠倒+排序`.
- Implemented identically in the Rust solver
  (`src/solver_handlers/numeric_list/mod.rs`) and the browser worker mirror
  (`src/web/formal_ai_worker.js`) so both runtimes inherit the prior list and
  reverse it; covered by `tests/integration/issue_427_invert_sort.rs`, the
  `operation_vocabulary_reverse_sort_matches_invert_phrasings` source test, and
  the `experiments/issue-427-worker-invert-sort-parity.mjs` cross-runtime check.
