---
bump: minor
---

### Added
- Budget-driven random and evolutionary search in solution synthesis (issue #662, F4). When reuse and rule reasoning produce no candidate, step 7 now samples and evolves compositions of the known numbers against the step-6 generated tests until it reaches the target or exhausts the compute budget.
- `compute_budget` knob on `SolverConfig`, wired through the `FORMAL_AI_COMPUTE_BUDGET` environment variable and the `--compute-budget` CLI flag, counting candidate evaluations.
- Atomic `search:` trace events recording each generation — `search:problem:{target,numbers,ops}`, `search:budget`, `search:test:{each_number_once,only_operators,evaluates_to}`, `search:random:{sampled,best_diff}`, `search:evolutionary:{generation,best_diff}`, `search:candidate:{phase,evaluations,expression}`, `search:solution`, `search:exhausted:{evaluations,budget,best_diff}`; on budget exhaustion the honest unknown-reasoning reply keeps the search evidence attached. Each event carries a single slug/value pair, so no user-facing prose is hardcoded in the trace (R379).
- A self-authored, search-only benchmark source (base + held-out variant) in the industry suite, raising `minimum_pass_count` from 10 to 12.
- `search:skill` proposal-only auto-learning event: a solved composition is recorded as a `candidate_skill` in status `proposed`, never promotable without review (`search:skill:promotable` is always `0`), mirroring the skill-accumulation ledger (R21/R340, C3/R13).
- Grounded meta-algorithm recipe `data/meta/budget-search-recipe.lino`, pinned by `tests/unit/specification/budget_search_meta_algorithm.rs` and documented in `docs/meta-algorithm.md`, so the nine-step stage always describes how the running code was produced.

### Changed
- Search recognition is now grounded entirely in the seed lexicon (issue #386): the operand framing, search cue, and target marker are read by semantic role from `data/seed/meanings-search.lino` instead of hardcoded per-language phrase tables, so the reach-a-target class spans en/ru/hi/zh (and any language whose surfaces are added to the seed) without touching Rust.
- The operator toolbox is derived from the seed `arithmetic_operator_word` vocabulary via `seed::Lexicon::arithmetic_operators`, generalising the search from `+ - *` to include division and modulo (with a division/modulo-by-zero guard) the moment the seed lists them.
- The budget-search reply is looked up from the seed knowledge base by the prompt's detected language (intent `budget_search_solution`, localized en/ru/hi/zh in `data/seed/multilingual-responses.lino`) instead of a hardcoded English string, so a Russian puzzle is answered in Russian (R379: data is the interface).
