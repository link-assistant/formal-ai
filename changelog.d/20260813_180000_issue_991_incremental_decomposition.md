---
bump: minor
---

### Added
- Failure-driven splitting: `TaskExecutor` gained a `split` hook, so a failed task can be shrunk from its own failure instead of from a plan made before any evidence existed. `formal_ai::task_decomposition::SplittingExecutor` answers that hook with the repository's own `decompose_task`, one level per split, and records every split with the failure that justified it. The controller refuses a child that repeats its parent, bounds splitting with `DEFAULT_SPLIT_DEPTH_BOUND`, and `solve_recursively_within` lets a caller pick another bound (zero reproduces the previous plan-driven protocol exactly).
- `formal-ai agent dispatch --incremental` runs that protocol against external agent CLIs: the whole task is attempted first, only a failure is split, a passing attempt's effects are applied to the workspace before the next attempt starts, and an irreducible failure escalates to the next CLI in `--cli` instead of stopping. The report carries an `incremental` trace of every attempt, split, and blocked task; the exit status reflects the root task only.
- Every blocked task becomes a review request, mirrored to `proposals.lino` next to the report: the task, every CLI that tried it, the evidence each attempt produced, and the status `human_review_required`. A run cannot approve its own extension, so this is the same gate a learned decomposition strategy passes through.

### Changed
- `RecursiveRun` now reports `split_applied`, `split_depth_reached()`, and `blocked_leaves()`, and the review-gated learning path reads blocked leaves from the run instead of walking the tree a second time.
