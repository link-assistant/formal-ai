---
bump: minor
---

### Added
- Budget-driven random and evolutionary search in solution synthesis (issue #662, F4). When reuse and rule reasoning produce no candidate, step 7 now samples and evolves compositions of the known numbers against the step-6 generated tests until it reaches the target or exhausts the compute budget.
- `compute_budget` knob on `SolverConfig`, wired through the `FORMAL_AI_COMPUTE_BUDGET` environment variable and the `--compute-budget` CLI flag, counting candidate evaluations.
- `search:` trace events (`search:problem`, `search:budget`, `search:test`, `search:random`, `search:evolutionary`, `search:candidate`, `search:solution`, `search:exhausted`) recording each generation; on budget exhaustion the honest unknown-reasoning reply keeps the search evidence attached.
- A self-authored, search-only benchmark source (base + held-out variant) in the industry suite, raising `minimum_pass_count` from 10 to 12.
- `search:skill` proposal-only auto-learning event: a solved composition is recorded as a `candidate_skill` in status `proposed`, never promotable without review (`search:skill:promotable` is always `0`), mirroring the skill-accumulation ledger (R21/R340, C3/R13).
- Grounded meta-algorithm recipe `data/meta/budget-search-recipe.lino`, pinned by `tests/unit/specification/budget_search_meta_algorithm.rs` and documented in `docs/meta-algorithm.md`, so the nine-step stage always describes how the running code was produced.

### Changed
- Search recognition is now grounded entirely in the seed lexicon (issue #386): the operand framing, search cue, and target marker are read by semantic role from `data/seed/meanings-search.lino` instead of hardcoded per-language phrase tables, so the reach-a-target class spans en/ru/hi/zh (and any language whose surfaces are added to the seed) without touching Rust.
- The operator toolbox is derived from the seed `arithmetic_operator_word` vocabulary via `seed::Lexicon::arithmetic_operators`, generalising the search from `+ - *` to include division and modulo (with a division/modulo-by-zero guard) the moment the seed lists them.
