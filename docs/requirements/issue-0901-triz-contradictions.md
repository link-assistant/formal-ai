## Issue #901 TRIZ Contradictions

Trade-offs are represented as inspectable links between satisfying candidates.
The selected point is derived from criterion-bearing requirement clauses; it is
never an unexplained default at fifty percent. The reusable principles remain
seed data so they can be forgotten and rediscovered without changing code.

| ID | Requirement | Status / evidence |
| --- | --- | --- |
| R901-1 | Represent each technical contradiction as a link between candidates and expose a selection value from 0 to 1. | Implemented by `ContradictionLink.selection_basis_points` and its Links Notation rendering in `src/selection_heuristics.rs`; covered by `tests/unit/issue_1138_selection_heuristics.rs`. |
| R901-2 | Derive the selection point from explicit criteria instead of silently choosing a midpoint. | Implemented by `contradictions_in`: requirement clauses or a cited seed record supply `ContradictionDerivation`; otherwise the result is `Underivable` and remains unresolved. |
| R901-3 | Make contradiction resolution a registry-selected ranking method rather than a separate hard-coded route. | Implemented by the `rank` role in `data/meta/selection-heuristics.lino` and `TrizRanker`; registry specification tests keep heuristics out of route dispatch. |
| R901-4 | Preserve the 40 inventive principles and four separation principles as source-linked, forgettable knowledge. | Implemented by `data/seed/triz-principles.lino`; the forget-and-rediscover regression is in `tests/unit/issue_1138_selection_heuristics.rs`. |
| R901-5 | Validate the mechanism on at least 20 distinct tasks. | Implemented by the 20-case, five-language corpus in `data/benchmarks/selection-triz.lino`; every declared relation is derived and checked by `the_twenty_task_triz_corpus_derives_each_declared_selection_relation`. |
