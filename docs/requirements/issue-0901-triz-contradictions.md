## Issue #901 TRIZ Contradictions

Trade-offs are represented as inspectable links between satisfying candidates.
The selected point is derived from criterion-bearing requirement clauses; it is
never an unexplained default at fifty percent. The reusable principles remain
seed data so they can be forgotten and rediscovered without changing code.

| ID | Requirement | Status / evidence |
| --- | --- | --- |
| R901-1 | Represent each technical contradiction as a link between candidates and expose a selection value from 0 to 1. | Implemented by `ContradictionLink.selection_basis_points` and its Links Notation rendering in `rust/src/selection_heuristics.rs`; covered by `rust/tests/unit/issue_1138_selection_heuristics.rs`. |
| R901-2 | Derive the selection point from explicit criteria instead of silently choosing a midpoint. | Implemented by `contradictions_in`: requirement clauses or a cited seed record supply `ContradictionDerivation`; otherwise the result is `Underivable` and remains unresolved. |
| R901-3 | Make contradiction resolution a registry-selected ranking method rather than a separate hard-coded route. | Delivered, 2026-10-08. `data/meta/selection-heuristics.lino` now declares `heuristic_triz` at role `rank`, order 3, with `applies_when contradiction_detected`. `draft_portfolio.rs::rank_passing_drafts` and `algorithm_discovery/ranking.rs::rank_survivors` both call `contradictions_in` to detect the situation and resolve their ranker through `heuristics_for(HeuristicRole::Rank, situation)`, selecting `TrizRanker` at order 3 when a contradiction is detected and keeping `LeastActionRanker` for every other situation. The JavaScript twin `js/agentic/crate/selection_heuristics_triz.mjs` mirrors `contradictions_in`, `TrizRanker`, and the registry selection (`rankWithHeuristic`, `situationFor`). Pinned by `rust/tests/unit/specification/triz_contradictions.rs` (seeded row, registry selection, ranker disagreement on the contradicted pair) and `rust/tests/web/issue-0901-triz-registry.test.mjs` (JS parity). |
| R901-4 | Preserve the 40 inventive principles and four separation principles as source-linked, forgettable knowledge. | Implemented by `data/seed/triz-principles.lino`; the forget-and-rediscover regression is in `rust/tests/unit/issue_1138_selection_heuristics.rs`. |
| R901-5 | Validate the mechanism on at least 20 distinct tasks. | Implemented by the 20-case, five-language corpus in `data/benchmarks/selection-triz.lino`; every declared relation is derived and checked by `the_twenty_task_triz_corpus_derives_each_declared_selection_relation`. Pinned by `rust/tests/unit/issue_1138_selection_heuristics.rs`. |
