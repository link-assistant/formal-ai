## Issue #1138 Selection Heuristics

Candidate selection is associative, deterministic, and subordinate to
satisfaction. The contract is covered by
`tests/unit/issue_1138_selection_heuristics.rs`.

| ID | Requirement | Status / evidence |
| --- | --- | --- |
| R1138-B12-1 | Selection heuristics live in the method registry as link data, are never route targets, and their precedence changes by data edit. | Covered by `tests/unit/issue_1138_selection_heuristics.rs`. |
| R1138-B12-2 | No heuristic may rank an unsatisfying candidate above a satisfying one, and no ranking key depends on wall-clock time. | Covered by `tests/unit/issue_1138_selection_heuristics.rs`. |
| R1138-B12-3 | An empty heuristic table falls back to deterministic identity ordering and records that fallback. | Covered by `tests/unit/issue_1138_selection_heuristics.rs`. |
| R1138-B12-4 | The non-binary work-unit count is measured, recorded, and strictly decreasing. | Covered by `tests/unit/issue_1138_selection_heuristics.rs`. |
