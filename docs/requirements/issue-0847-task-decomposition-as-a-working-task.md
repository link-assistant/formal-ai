## Issue #847 Task Decomposition As A Working Task

Issue [#847](https://github.com/link-assistant/formal-ai/issues/847) (E57)
records that both halves of decomposition were refused: "split this task into
subtasks" and "is this task atomic?" fell through to the unknown-prompt
fallback or misrouted to `write_program`. PR
[#857](https://github.com/link-assistant/formal-ai/pull/857) makes decomposition
a working task — one recursion seen from two sides, recognised from seed
surfaces rather than per-language phrase lists (issue #386), with the atomicity
judgement as its base case.

| ID | Requirement | Status |
| --- | --- | --- |
| R847-1 | Decomposition must be a recognised intent in every supported language, never the unknown-prompt fallback, with the surfaces in seed data and no per-language phrase list in Rust. | `data/seed/meanings-decomposition.lino` carries the splitting, sub-task, atomicity, first-step and enumeration roles for en/ru/hi/zh; `src/solver_handlers/task_decomposition.rs::classify` names only the roles. Covered by `splitting_is_recognised_in_every_supported_language` and `decomposition_is_recognised_in_every_supported_language`. |
| R847-2 | Splitting a task must yield children that each carry an observable completion criterion. | `is_checkable` in `src/task_decomposition.rs` rejects unobservable fragments such as "understand the codebase" and merges them into a sibling; covered by `every_child_has_an_observable_completion_criterion` and `an_unobservable_fragment_never_becomes_a_child`. |
| R847-3 | "Is this task atomic?" must be answerable standalone and must be the recursion's base case. | `DecompositionQuestion::Atomicity` answers from the same `decompose_task` call the split answer uses, so the two cannot disagree; covered by `atomicity_is_recognised_in_every_supported_language` and `asking_whether_a_task_is_atomic_is_answered_standalone`. |
| R847-4 | The recursion must terminate: every leaf atomic, or `max_decomposition_depth` reached and reported rather than hidden. | `decompose_task` records an `AtomicityReason` per leaf and `Decomposition::depth_bound_reached` drives the reported note and the per-row markers from one test; covered by `recursion_terminates_with_atomic_leaves_or_a_reported_depth_bound` and `the_configured_depth_bound_is_reported_in_the_answer`. |
| R847-5 | Every sub-task must be a first-class inspectable `sub_impulse:` event, reusing the existing structure. | `record_task_decomposition` appends one `sub_impulse` event per sub-task, so each surfaces as a `sub_impulse:<id>` evidence link; covered by `every_sub_task_surfaces_as_a_sub_impulse_event` and `every_sub_task_is_an_inspectable_sub_impulse_event`. |
| R847-6 | The decomposition must be deterministic: the same task and configuration always produce the same result. | The splitter is a pure function of the task text and the depth bound, with no clock, randomness or ordering by hash; covered by `decomposition_is_deterministic_for_a_given_config` and `decomposition_is_deterministic`. |
| R847-7 | A real corpus task must decompose into children a human agrees are smaller and jointly sufficient. | Covered by `a_real_corpus_task_splits_into_smaller_jointly_sufficient_children`, which splits this repository's own `experiments/` code-change-detector task into its two edits. |
| R847-8 | The spectrum must run from a GitHub issue down to a single atomic edit, with regression coverage in the specification suites in all four languages. | `the_spectrum_runs_from_issue_to_atomic_edit` pins both ends of the ladder; `tests/unit/specification/task_decomposition.rs` holds the four-language specifications and `tests/unit/issue_847_task_decomposition.rs` the reproduction cases. |
