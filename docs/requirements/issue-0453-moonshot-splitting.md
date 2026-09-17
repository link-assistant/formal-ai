## Issue #453 Moonshot Task Splitting and Approach Provenance

Large tasks use the same observable binary-decomposition contract as ordinary
tasks. Splitting may group clauses but may not discard them or pretend an
underivable split is atomic. Independently discovered approaches are combined
conservatively while preserving the first source in the supplied history.

| ID | Requirement | Status / evidence |
| --- | --- | --- |
| R453-M1 | Recursively split every non-leaf task into exactly two checkable children when a grounded split can be derived. | Implemented by `balanced_split`, `TaskSplitter`, the `split` heuristic role, and the binary work-unit ratchet; covered in five languages by `tests/unit/issue_1138_selection_heuristics.rs`. |
| R453-M2 | Preserve every source segment exactly once while balancing the two sides. | Implemented by `BinarySplit` spans and weights; exact coverage and imbalance regressions are in the issue #1138 selection tests. |
| R453-M3 | When a single-clause task cannot yet be grounded into sub-obligations, report the split as underivable instead of certifying it atomic. | Implemented by `SplitRefusal::Underivable`; the five-language moonshot regression keeps this boundary explicit. |
| R453-M4 | Combine distinct approaches, remove semantic duplicates, and retain the first source of every duplicated idea in history order. | Implemented by `combine_approaches`, which delegates equivalence to the shared source-synthesis deduplicator and exposes `first_source` plus all sources; covered by `equivalent_approaches_merge_without_losing_their_first_historical_source`. |
