## Issue #802 Refutation-First Hypothesis Search

The 2-4-6 lesson is implemented as a domain-independent experiment chooser:
maintain live hypotheses, try to refute them, prefer the probe with the smallest
worst-case survivor set, and report uncertainty when evidence cannot decide.

| ID | Requirement | Status / evidence |
| --- | --- | --- |
| R802-1 | Represent hypotheses, discriminating experiments, observations, and the live hypothesis space as inspectable data. | Implemented by `SearchHypothesis`, `Experiment`, `HypothesisSpace`, and their Links Notation records in `rust/src/selection_heuristics.rs`; pinned by `rust/tests/unit/specification/refutation_search.rs` (`observing_a_result_kills_every_contradicted_hypothesis_and_names_the_experiment`, `every_elimination_is_backed_by_an_evidence_record`). |
| R802-2 | Try to disprove a hypothesis first and select experiments that minimize the worst-case survivors. | Implemented by `attempts_refutation_of`, `worst_case_survivors`, and `RefutationSearch`; covered by the issue #1138 selection tests in `rust/tests/unit/specification/refutation_search.rs` (`the_chosen_experiment_minimizes_worst_case_survivors`, `a_refuting_probe_is_preferred_over_a_confirming_one_at_equal_power`). |
| R802-3 | Use refutation-first search before stochastic sampling when the solver has a discriminating probe. | Implemented by `run_refutation_search` in `rust/src/solver_search.rs`; sampling remains an explicit fallback when no probe discriminates. Pinned by `rust/tests/unit/budget_search.rs::budget_search_solves_reachability_under_sufficient_budget`, which requires the solved trace to pass through the registry-backed refutation chooser with evidence-backed eliminations. |
| R802-4 | Feed the search verdict into the reasoning standard and remain honestly undecided when neither side is proved. | Implemented by `hypothesis_search_verdict` in `rust/src/reasoning_standard/mod.rs`; `SearchVerdict::NotConfirmedNotRefuted` preserves the blockers. Pinned by `rust/tests/unit/issue_1073_reasoning_standard.rs`, which requires an unprobed episode to reach `not_confirmed_not_refuted` with the `hypothesis_search:no_refutation_attempted` blocker. |
