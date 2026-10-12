# Issue #491 Traceability Rows

Rows for a shard `docs/requirements/issue-491-least-action.md`, kept
here for the maintainer to merge into `docs/requirements-traceability.md`
(the shared table is not edited by this change).

| ID | Shard | Delivered | Automated test | Manual confirmation |
| --- | --- | --- | --- | --- |
| R491-1 | docs/requirements/issue-491-least-action.md | 2026-09-30 (this pull request) | rust/tests/unit/issue_491_least_action.rs | not yet confirmed |
| R491-2 | docs/requirements/issue-491-least-action.md | 2026-09-30 (this pull request) | rust/tests/unit/issue_491_least_action.rs | not yet confirmed |
| R491-3 | docs/requirements/issue-491-least-action.md | 2026-09-30 (this pull request) | rust/tests/unit/issue_491_least_action.rs | not yet confirmed |
| R491-4 | docs/requirements/issue-491-least-action.md | 2026-09-30 (this pull request) | rust/tests/unit/issue_491_least_action.rs | not yet confirmed |

Requirement mapping:

- R491-1 — balanced binary splits with a 1, 2, 4, 8 abstraction ladder
  (`the_highest_abstraction_is_always_one_two_four_eight`).
- R491-2 — optimize the total number of smallest subtasks; atomic
  subtasks with known solutions costed, open work listed uncosted
  (`the_plan_counts_smallest_subtasks_and_leaves_open_work_uncosted`,
  `a_plan_with_fewer_smallest_subtasks_is_less_action`).
- R491-3 — score generated solutions by least action: shortest
  steps/path first, then code, compute, memory; entire input range
  required (`solutions_rank_by_least_action_steps_first`,
  `a_millisecond_saved_by_an_extra_step_still_loses`,
  `the_shortest_code_that_fails_the_input_range_never_ranks`).
- R491-4 — evaluation on time, computation, memory with satisfaction in
  least resources (ActionCost dimensions; lexicographic gate; residual:
  satisfaction signal pending, noted in the case study).
