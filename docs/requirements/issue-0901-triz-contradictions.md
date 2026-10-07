## Issue #901 TRIZ Contradictions

Trade-offs are represented as inspectable links between satisfying candidates.
The selected point is derived from criterion-bearing requirement clauses; it is
never an unexplained default at fifty percent. The reusable principles remain
seed data so they can be forgotten and rediscovered without changing code.

| ID | Requirement | Status / evidence |
| --- | --- | --- |
| R901-1 | Represent each technical contradiction as a link between candidates and expose a selection value from 0 to 1. | Implemented by `ContradictionLink.selection_basis_points` and its Links Notation rendering in `rust/src/selection_heuristics.rs`; covered by `rust/tests/unit/issue_1138_selection_heuristics.rs`. |
| R901-2 | Derive the selection point from explicit criteria instead of silently choosing a midpoint. | Implemented by `contradictions_in`: requirement clauses or a cited seed record supply `ContradictionDerivation`; otherwise the result is `Underivable` and remains unresolved. |
| R901-3 | Make contradiction resolution a registry-selected ranking method rather than a separate hard-coded route. | Partial, re-checked 2026-10-08. In place: `TrizRanker` is a `CandidateRanker` beside `LeastActionRanker` (a ranking method, not a route), and `rust/tests/unit/specification/selection_heuristics.rs` keeps heuristics out of route dispatch. Open: `data/meta/selection-heuristics.lino` declares no `triz` row in the `rank` role (only `least_action` and `least_resources`), and no caller selects `TrizRanker` — `draft_portfolio.rs` and `algorithm_discovery/ranking.rs` rank with `LeastActionRanker` directly — so contradiction resolution is not yet registry-selected; plan 12's leaf "seed the `rank` role at order 3" was ticked without the row landing. Closing it needs the seeded row (`applies_when contradiction_detected`), the ranking call sites resolving their ranker through `heuristics_for(HeuristicRole::Rank, situation)`, and the JavaScript twin of `contradictions_in` and `TrizRanker` in the agentic crate (JavaScript first, R997). |
| R901-4 | Preserve the 40 inventive principles and four separation principles as source-linked, forgettable knowledge. | Implemented by `data/seed/triz-principles.lino`; the forget-and-rediscover regression is in `rust/tests/unit/issue_1138_selection_heuristics.rs`. |
| R901-5 | Validate the mechanism on at least 20 distinct tasks. | Implemented by the 20-case, five-language corpus in `data/benchmarks/selection-triz.lino`; every declared relation is derived and checked by `the_twenty_task_triz_corpus_derives_each_declared_selection_relation`. Pinned by `rust/tests/unit/issue_1138_selection_heuristics.rs`. |
