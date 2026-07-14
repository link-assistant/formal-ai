---
bump: minor
---

### Added
- Budget-driven random and evolutionary search in solution synthesis (issue #662, F4). When reuse and rule reasoning produce no candidate, step 7 now samples and evolves compositions of the known numbers against the step-6 generated tests until it reaches the target or exhausts the compute budget.
- `compute_budget` knob on `SolverConfig`, wired through the `FORMAL_AI_COMPUTE_BUDGET` environment variable and the `--compute-budget` CLI flag, counting candidate evaluations.
- `search:` trace events (`search:problem`, `search:budget`, `search:test`, `search:random`, `search:evolutionary`, `search:candidate`, `search:solution`, `search:exhausted`) recording each generation; on budget exhaustion the honest unknown-reasoning reply keeps the search evidence attached.
- A self-authored, search-only benchmark source (base + held-out variant) in the industry suite, raising `minimum_pass_count` from 10 to 12.
