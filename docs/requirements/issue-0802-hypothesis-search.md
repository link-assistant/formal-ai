## Issue #802 Refutation-First Hypothesis Search

The 2-4-6 lesson is implemented as a domain-independent experiment chooser:
maintain live hypotheses, try to refute them, prefer the probe with the smallest
worst-case survivor set, and report uncertainty when evidence cannot decide.

| ID | Requirement | Status / evidence |
| --- | --- | --- |
| R802-1 | Represent hypotheses, discriminating experiments, observations, and the live hypothesis space as inspectable data. | Implemented by `SearchHypothesis`, `Experiment`, `HypothesisSpace`, and their Links Notation records in `src/selection_heuristics.rs`. |
| R802-2 | Try to disprove a hypothesis first and select experiments that minimize the worst-case survivors. | Implemented by `attempts_refutation_of`, `worst_case_survivors`, and `RefutationSearch`; covered by the issue #1138 selection tests. |
| R802-3 | Use refutation-first search before stochastic sampling when the solver has a discriminating probe. | Implemented by `run_refutation_search` in `src/solver_search.rs`; sampling remains an explicit fallback when no probe discriminates. |
| R802-4 | Feed the search verdict into the reasoning standard and remain honestly undecided when neither side is proved. | Implemented by `hypothesis_search_verdict` in `src/reasoning_standard/mod.rs`; `SearchVerdict::NotConfirmedNotRefuted` preserves the blockers. |
