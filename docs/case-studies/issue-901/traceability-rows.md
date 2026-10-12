# Issue #901 Traceability Rows

Rows for a shard `docs/requirements/issue-901-triz-paradox-resolution.md`,
kept here for the maintainer to merge into
`docs/requirements-traceability.md` (the shared table is not edited by
this change).

| ID | Shard | Delivered | Automated test | Manual confirmation |
| --- | --- | --- | --- | --- |
| R901-1 | docs/requirements/issue-901-triz-paradox-resolution.md | 2026-09-30 (this pull request) | rust/tests/unit/issue_901_triz_solver.rs | not yet confirmed |
| R901-2 | docs/requirements/issue-901-triz-paradox-resolution.md | 2026-09-30 (this pull request) | rust/tests/unit/issue_901_triz_solver.rs | not yet confirmed |
| R901-3 | docs/requirements/issue-901-triz-paradox-resolution.md | 2026-09-30 (this pull request) | rust/tests/unit/issue_901_triz_solver.rs | not yet confirmed |
| R901-4 | docs/requirements/issue-901-triz-paradox-resolution.md | 2026-09-30 (this pull request) | rust/tests/unit/issue_901_triz_solver.rs | not yet confirmed |

Requirement mapping:

- R901-1 — contradictions as links with a 0-1 value chosen from the
  requirements (already computed in selection_heuristics; the answer now
  states it: "no default 50 %" assertion).
- R901-2 — collect the other common paradox-resolution ways as seed
  data (twelve `triz_resolution_family` records).
- R901-3 — top-20 benchmark corpus, replayed against prompts
  (twenty `triz_benchmark_task` records; umbrella and pill-coating
  precedent tests).
- R901-4 — put in use: the user-facing entry `handle_triz` reading the
  seed, with cues, logging, and the localized template.
