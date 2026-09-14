node_path=1.1.1.1.2

The `grep` command completed. Output:

```text
Found 100 matches
/tmp/tmp.ycfvUjtvtl/REQUIREMENTS.md:
  Line 1602: | R847-1 | Decomposition must be a recognised intent in every supported language, never the unknown-prompt fallback, with the surfaces in seed data and no per-language phrase list in Rust. | `data/seed/meanings-decomposition.lino` carries the splitting, sub-task, atomicity, first-step and enumeration roles for en/ru/hi/zh; `src/solver_handlers/task_decomposition.rs::classify` names only the roles. Covered by `splitting_is_recognised_in_every_supported_language` and `decomposition_is_recognised_in_every_supported_language`. |
  Line 1603: | R847-2 | Splitting a task must yield children that each carry an observable completion criterion. | `is_checkable` in `src/task_decomposition.rs` rejects unobservable fragments such as "understand the codebase" and merges them into a sibling; covered by `every_child_has_an_observable_completion_criterion` and `an_unobservable_fragment_never_becomes_a_child`. |
  Line 1606: | R847-5 | Every sub-task must be a first-class inspectable `sub_impulse:` event, reusing the existing structure. | `record_task_decomposition` appends one `sub_impulse` event per sub-task, so each surfaces as a `sub_impulse:<id>` evidence link; covered by `every_sub_task_surfaces_as_a_sub_impulse_event` and `every_sub_task_is_an_inspectable_sub_impulse_event`. |
  Line 1609: | R847-8 | The spectrum must run from a GitHub issue down to a single atomic edit, with regression coverage in the specification suites in all four languages. | `the_spectrum_runs_from_issue_to_atomic_edit` pins both ends of the ladder; `tests/unit/specification/task_decomposition.rs` holds the four-language specifications and `tests/unit/issue_847_task_decomposition.rs` the reproduction cases. |
  Line 2180: | R991-9 | A task too large for one agent CLI session is split from its own recorded failure, not from a plan committed to before any evidence existed, and an irreducible failure is reported for review instead of retried forever. | Implemented: `TaskExecutor::split` (`src/recursive_execution.rs`) is answered by `formal_ai::task_decomposition::SplittingExecutor` with the repository's own `decompose_task`, one level per split and bounded by `DEFAULT_SPLIT_DEPTH_BOUND`; a child that repeats its parent is refused. `formal-ai agent dispatch --incremental` (`src/orchestration/incremental.rs`) runs that protocol against real CLIs, applies a passing attempt's effects before the next attempt starts, escalates an irreducible failure to the next `--cli`, and reports every attempt, split, and blocked task in the `incremental` trace. Pinned by `tests/unit/issue_991_incremental_decomposition.rs` and, through real processes, `tests/integration/issue_991_incremental_dispatch.rs`. |

/tmp/tmp.ycfvUjtvtl/examples/issue_1066_decomposition_probe.rs:
  Line 5:     let decomposition = formal_ai::task_decomposition::decompose_task(&task, 4);
  Line 14:         formal_ai::task_decomposition::split_once_checkable(&task)

/tmp/tmp.ycfvUjtvtl/examples/dump_task_decomposition.rs:
  Line 3: //! Usage: `cargo run --example dump_task_decomposition -- "<task>" [max_depth]`
  Line 5: use formal_ai::task_decomposition::{decompose_task, is_checkable};

/tmp/tmp.ycfvUjtvtl/docs/requirements-traceability.md:
  Line 654: | R847-1 | 1538 | PR #857 (issue #847) | issue-level coverage (not row-pinned): tests/unit/issue_847_task_decomposition.rs | not yet confirmed |
  Line 655: | R847-2 | 1539 | PR #857 (issue #847) | issue-level coverage (not row-pinned): tests/unit/issue_847_task_decomposition.rs | not yet confirmed |
  Line 656: | R847-3 | 1540 | PR #857 (issue #847) | issue-level coverage (not row-pinned): tests/unit/issue_847_task_decomposition.rs | not yet confirmed |
  Line 657: | R847-4 | 1541 | PR #857 (issue #847) | issue-level coverage (not row-pinned): tests/unit/issue_847_task_decomposition.rs | not yet confirmed |
  Line 658: | R847-5 | 1542 | PR #857 (issue #847) | issue-level coverage (not row-pinned): tests/unit/issue_847_task_decomposition.rs | not yet confirmed |
  Line 659: | R847-6 | 1543 | PR #857 (issue #847) | issue-level coverage (not row-pinned): tests/unit/issue_847_task_decomposition.rs | not yet confirmed |
  Line 660: | R847-7 | 1544 | PR #857 (issue #847) | issue-level coverage (not row-pinned): tests/unit/issue_847_task_decomposition.rs | not yet confirmed |
  Line 661: | R847-8 | 1545 | PR #857 (issue #847) | tests/unit/specification/task_decomposition.rs; tests/unit/issue_847_task_decomposition.rs | not yet confirmed |

/tmp/tmp.ycfvUjtvtl/examples/issue_1066_checkable_probe.rs:
  Line 8:             formal_ai::task_decomposition::is_checkable(&segment)

/tmp/tmp.ycfvUjtvtl/CHANGELOG.md:
  Line 65: - Failure-driven splitting: `TaskExecutor` gained a `split` hook, so a failed task can be shrunk from its own failure instead of from a plan made before any evidence existed. `formal_ai::task_decomposition::SplittingExecutor` answers that hook with the repository's own `decompose_task`, one level per split, and records every split with the failure that justified it. The controller refuses a child that repeats its parent, bounds splitting with `DEFAULT_SPLIT_DEPTH_BOUND`, and `solve_recursively_within` lets a caller pick another bound (zero reproduces the previous plan-driven protocol exactly).

/tmp/tmp.ycfvUjtvtl/examples/issue_991_incremental_decomposition.rs:
  Line 15: use formal_ai::task_decomposition::SplittingExecutor;

/tmp/tmp.ycfvUjtvtl/docs/requirements/issue-0847-task-decomposition-as-a-working-task.md:
  Line 14: | R847-1 | Decomposition must be a recognised intent in every supported language, never the unknown-prompt fallback, with the surfaces in seed data and no per-language phrase list in Rust. | `data/seed/meanings-decomposition.lino` carries the splitting, sub-task, atomicity, first-step and enumeration roles for en/ru/hi/zh; `src/solver_handlers/task_decomposition.rs::classify` names only the roles. Covered by `splitting_is_recognised_in_every_supported_language` and `decomposition_is_recognised_in_every_supported_language`. |
  Line 15: | R847-2 | Splitting a task must yield children that each carry an observable completion criterion. | `is_checkable` in `src/task_decomposition.rs` rejects unobservable fragments such as "understand the codebase" and merges them into a sibling; covered by `every_child_has_an_observable_completion_criterion` and `an_unobservable_fragment_never_becomes_a_child`. |
  Line 18: | R847-5 | Every sub-task must be a first-class inspectable `sub_impulse:` event, reusing the existing structure. | `record_task_decomposition` appends one `sub_impulse` event per sub-task, so each surfaces as a `sub_impulse:<id>` evidence link; covered by `every_sub_task_surfaces_as_a_sub_impulse_event` and `every_sub_task_is_an_inspectable_sub_impulse_event`. |
  Line 21: | R847-8 | The spectrum must run from a GitHub issue down to a single atomic edit, with regression coverage in the specification suites in all four languages. | `the_spectrum_runs_from_issue_to_atomic_edit` pins both ends of the ladder; `tests/unit/specification/task_decomposition.rs` holds the four-language specifications and `tests/unit/issue_847_task_decomposition.rs` the reproduction cases. |

/tmp/tmp.ycfvUjtvtl/docs/requirements/issue-0991-dynamic-multi-source-how-to-synthesis.md:
  Line 25: | R991-9 | A task too large for one agent CLI session is split from its own recorded failure, not from a plan committed to before any evidence existed, and an irreducible failure is reported for review instead of retried forever. | Implemented: `TaskExecutor::split` (`src/recursive_execution.rs`) is answered by `formal_ai::task_decomposition::SplittingExecutor` with the repository's own `decompose_task`, one level per split and bounded by `DEFAULT_SPLIT_DEPTH_BOUND`; a child that repeats its parent is refused. `formal-ai agent dispatch --incremental` (`src/orchestration/incremental.rs`) runs that protocol against real CLIs, applies a passing attempt's effects before the next attempt starts, escalates an irreducible failure to the next `--cli`, and reports every attempt, split, and blocked task in the `incremental` trace. Pinned by `tests/unit/issue_991_incremental_decomposition.rs` and, through real processes, `tests/integration/issue_991_incremental_dispatch.rs`. |

/tmp/tmp.ycfvUjtvtl/src/engine.rs:
  Line 128:     /// so ([`crate::task_decomposition::Decomposition::unenumerable_reason`]);

/tmp/tmp.ycfvUjtvtl/src/solver_dispatch.rs:
  Line 37:     try_summarization_request, try_task_decomposition_with_depth, try_text_manipulation,
  Line 140:     "task_decomposition",
  Line 217:         "task_decomposition" => try_task_decomposition_with_depth(

/tmp/tmp.ycfvUjtvtl/src/intent_formalization/prompt_relevants.rs:
  Line 51:             "handler:task_decomposition",
  Line 52:             crate::solver_handlers::looks_like_task_decomposition(normalized),

/tmp/tmp.ycfvUjtvtl/src/recursive_execution.rs:
  Line 15: //! honest move. [`crate::task_decomposition::SplittingExecutor`] wires the

/tmp/tmp.ycfvUjtvtl/docs/case-studies/issue-920/self-hosting-authorship/decomposition.lino:
  Line 2:   record_type "task_decomposition"

/tmp/tmp.ycfvUjtvtl/docs/case-studies/issue-932/self-hosting-authorship/decomposition.lino:
  Line 2:   record_type "task_decomposition"

/tmp/tmp.ycfvUjtvtl/dev/log/issues/1014/pulls/1015/ci-logs/pushed-head-c5fae9d4/Coverage-31897604157.log:
  Line 6086: Code Coverage	Generate code coverage	2026-08-15T17:30:10.9480232Z test issue_847_task_decomposition::asking_whether_a_task_is_atomic_is_answered_standalone ... ok
  Line 6087: Code Coverage	Generate code coverage	2026-08-15T17:30:11.7579861Z test issue_847_task_decomposition::atomicity_is_answerable_in_every_supported_language ... ok
  Line 6088: Code Coverage	Generate code coverage	2026-08-15T17:30:14.1309975Z test issue_847_task_decomposition::a_composite_task_is_reported_as_not_atomic ... ok
  Line 6089: Code Coverage	Generate code coverage	2026-08-15T17:30:14.3541572Z test issue_847_task_decomposition::decomposition_is_recognised_in_every_supported_language ... ok
  Line 6090: Code Coverage	Generate code coverage	2026-08-15T17:30:14.3599382Z test issue_847_task_decomposition::same_task_agent_cli_authorship_is_preserved ... ok
  Line 6091: Code Coverage	Generate code coverage	2026-08-15T17:30:14.8674784Z test issue_847_task_decomposition::decomposition_is_deterministic ... ok
  Line 6093: Code Coverage	Generate code coverage	2026-08-15T17:30:16.1316747Z test issue_847_task_decomposition::every_sub_task_is_an_inspectable_sub_impulse_event ... ok
  Line 6096: Code Coverage	Generate code coverage	2026-08-15T17:30:16.3903090Z test issue_847_task_decomposition::splitting_a_coding_task_into_subtasks_is_answered_not_refused ... ok
  Line 6997: Code Coverage	Generate code coverage	2026-08-15T17:32:13.5870471Z test specification::method_registry::task_decomposition_has_one_configured_contextual_dispatch_path ... ok
  Line 7445: Code Coverage	Generate code coverage	2026-08-15T17:32:46.3739835Z test specification::task_decomposition::a_real_corpus_task_splits_into_smaller_jointly_sufficient_children ... ok
  Line 7446: Code Coverage	Generate code coverage	2026-08-15T17:32:46.7304971Z test specification::task_decomposition::a_depth_bounded_unsolved_root_is_not_reported_as_atomic ... ok
  Line 7447: Code Coverage	Generate code coverage	2026-08-15T17:32:46.7319774Z test specification::task_decomposition::agent_authored_contract_is_exact_and_controls_the_shipped_ledger ... ok
  Line 7448: Code Coverage	Generate code coverage	2026-08-15T17:32:46.7349718Z test specification::task_decomposition::an_atomic_task_yields_no_split ... ok
  Line 7449: Code Coverage	Generate code coverage	2026-08-15T17:32:46.7379398Z test specification::task_decomposition::an_unobservable_fragment_never_becomes_a_child ... ok
  Line 7450: Code Coverage	Generate code coverage	2026-08-15T17:32:46.7492505Z test specification::task_decomposition::a_single_clause_issue_is_not_misreported_as_an_atomic_operation ... ok
  Line 7452: Code Coverage	Generate code coverage	2026-08-15T17:32:47.1559786Z test specification::task_decomposition::every_child_has_an_observable_completion_criterion ... ok
  Line 7453: Code Coverage	Generate code coverage	2026-08-15T17:32:48.1088367Z test specification::task_decomposition::atomicity_is_recognised_in_every_supported_language ... ok
  Line 7454: Code Coverage	Generate code coverage	2026-08-15T17:32:48.3814379Z test specification::task_decomposition::every_sub_task_surfaces_as_a_sub_impulse_event ... ok
  Line 7455: Code Coverage	Generate code coverage	2026-08-15T17:32:48.3959922Z test specification::task_decomposition::recursion_terminates_with_atomic_leaves_or_a_reported_depth_bound ... ok
  Line 7456: Code Coverage	Generate code coverage	2026-08-15T17:32:48.7630175Z test specification::task_decomposition::recursive_execution_reuses_the_exact_inspected_tree ... ok
  Line 7457: Code Coverage	Generate code coverage	2026-08-15T17:32:48.8543237Z test specification::task_decomposition::failed_execution_can_propose_a_strategy_but_only_reviewed_green_learning_activates_it ... ok
  Line 7459: Code Coverage	Generate code coverage	2026-08-15T17:32:51.2190193Z test specification::task_decomposition::the_configured_depth_bound_is_reported_in_the_answer ... ok
  Line 7460: Code Coverage	Generate code coverage	2026-08-15T17:32:51.3410138Z test specification::task_decomposition::decomposition_is_deterministic_for_a_given_config ... ok
  Line 7462: Code Coverage	Generate code coverage	2026-08-15T17:32:51.5294588Z test specification::task_decomposition::the_inspected_tree_round_trips_and_changed_artifacts_are_rejected ... ok
  Line 7465: Code Coverage	Generate code coverage	2026-08-15T17:32:51.5689049Z test specification::task_decomposition::splitting_is_recognised_in_every_supported_language ... ok
  Line 7477: Code Coverage	Generate code coverage	2026-08-15T17:32:51.9300045Z test specification::task_decomposition::the_spectrum_runs_from_issue_to_atomic_edit ... ok

/tmp/tmp.ycfvUjtvtl/src/solver_handlers/modules.rs:
  Line 41: mod task_decomposition;

/tmp/tmp.ycfvUjtvtl/src/solver_handlers/mod.rs:
  Line 42: pub use task_decomposition::{looks_like_task_decomposition, try_task_decomposition_with_depth};

/tmp/tmp.ycfvUjtvtl/src/solver_handlers/task_decomposition.rs:
  Line 7: //! one call into [`crate::task_decomposition`]: the atomicity answer is the
  Line 27: use crate::task_decomposition::{record_task_decomposition, stated_task, Decomposition};
  Line 51: pub fn looks_like_task_decomposition(normalized: &str) -> bool {
  Line 61: pub fn try_task_decomposition_with_depth(
  Line 79:     log.append("task_decomposition:question", question.slug().to_owned());
  Line 80:     let decomposition = record_task_decomposition(log, &task, max_depth);
  Line 91:         "response:task_decomposition",
  Line 176:         AtomicityReason::DepthBound => "task_decomposition_unsplit_depth_bound",
  Line 177:         _ => "task_decomposition_single_need",
  Line 196:             "task_decomposition",
  Line 197:             seeded(language, "task_decomposition_atomic"),
  Line 202:             "task_decomposition",
  Line 208:         "task_decomposition_lead",
  Line 212:         "task_decomposition",
  Line 221:     let marker = response(language, "task_decomposition_depth_marker", "[depth bound]");
  Line 225:         lines.push(seeded(language, "task_decomposition_depth_bound"));

/tmp/tmp.ycfvUjtvtl/src/agentic_coding/task_structure.rs:
  Line 7: //! [`crate::solver_handlers::task_decomposition`], which has answered the three
  Line 17: //! ([`looks_like_task_decomposition`]), so every phrasing and every language the
  Line 39: use crate::solver_handlers::looks_like_task_decomposition;
  Line 67:     if !looks_like_task_decomposition(&normalize_prompt(task)) {

/tmp/tmp.ycfvUjtvtl/dev/log/issues/1014/pulls/1015/raw-data/all-diagnostic-candidates.tsv:
  Line 613: run-31884932348.log	4010	57ec938a43b4	CodeQL (rust) UNKNOWN STEP 2026-08-15T12:35:29.2572399Z [2026-08-15 12:35:29] [build-stdout] [2026-08-15 12:35:29] [build-stdout] ^[[33m WARN^[[0m /home/runner/work/formal-ai/formal-ai/examples/dump_task_decomposition.rs:18:14: macro expansion failed for '$crate::format_args_nl'
  Line 614: run-31884932348.log	4011	f911658735ea	CodeQL (rust) UNKNOWN STEP 2026-08-15T12:35:29.2576262Z [2026-08-15 12:35:29] [build-stdout] [2026-08-15 12:35:29] [build-stdout] ^[[33m WARN^[[0m /home/runner/work/formal-ai/formal-ai/examples/dump_task_decomposition.rs:19:14: macro expansion failed for '$crate::format_args_nl'
  Line 615: run-31884932348.log	4012	0cf5023a8d99	CodeQL (rust) UNKNOWN STEP 2026-08-15T12:35:29.2579819Z [2026-08-15 12:35:29] [build-stdout] [2026-08-15 12:35:29] [build-stdout] ^[[33m WARN^[[0m /home/runner/work/formal-ai/formal-ai/examples/dump_task_decomposition.rs:21:9: macro expansion failed for '$crate::format_args_nl'
  Line 616: run-31884932348.log	4013	863ddb37ea3e	CodeQL (rust) UNKNOWN STEP 2026-08-15T12:35:29.2583246Z [2026-08-15 12:35:29] [build-stdout] [2026-08-15 12:35:29] [build-stdout] ^[[33m WARN^[[0m /home/runner/work/formal-ai/formal-ai/examples/dump_task_decomposition.rs:25:18: macro expansion failed for '$crate::format_args_nl'
  Line 617: run-31884932348.log	4014	cc19bbeee9dd	CodeQL (rust) UNKNOWN STEP 2026-08-15T12:35:29.2586416Z [2026-08-15 12:35:29] [build-stdout] [2026-08-15 12:35:29] [build-stdout] ^[[33m WARN^[[0m /home/runner/work/formal-ai/formal-ai/examples/dump_task_decomposition.rs:29:13: macro expansion failed for '$crate::format_args_nl'
  Line 618: run-31884932348.log	4015	3769f24fc6c5	CodeQL (rust) UNKNOWN STEP 2026-08-15T12:35:29.2589907Z [2026-08-15 12:35:29] [build-stdout] [2026-08-15 12:35:29] [build-stdout] ^[[33m WARN^[[0m /home/runner/work/formal-ai/formal-ai/examples/dump_task_decomposition.rs:37:14: macro expansion failed for '$crate::format_args_nl'
  Line 4038: run-31884932348.log	7623	cbf98ea0091c	CodeQL (rust) UNKNOWN STEP 2026-08-15T12:35:32.7785429Z [2026-08-15 12:35:32] [build-stdout] [2026-08-15 12:35:32] [build-stdout] ^[[33m WARN^[[0m /home/runner/work/formal-ai/formal-ai/src/task_decomposition.rs:140:46: macro expansion failed for 'vec'
  Line 4039: run-31884932348.log	7624	4c579fbb66f0	CodeQL (rust) UNKNOWN STEP 2026-08-15T12:35:32.7792092Z [2026-08-15 12:35:32] [build-stdout] [2026-08-15 12:35:32] [build-stdout] ^[[33m WARN^[[0m /home/runner/work/formal-ai/formal-ai/src/task_decomposition.rs:233:21: macro expansion failed for 'format'
  Line 4040: run-31884932348.log	7625	2cac36b08e7e	CodeQL (rust) UNKNOWN STEP 2026-08-15T12:35:32.7794074Z [2026-08-15 12:35:32] [build-stdout] [2026-08-15 12:35:32] [build-stdout] ^[[33m WARN^[[0m /home/runner/work/formal-ai/formal-ai/src/task_decomposition.rs:225:21: macro expansion failed for 'format'
  Line 4041: run-31884932348.log	7626	3230bfa5e73a	CodeQL (rust) UNKNOWN STEP 2026-08-15T12:35:32.7798862Z [2026-08-15 12:35:32] [build-stdout] [2026-08-15 12:35:32] [build-stdout] ^[[33m WARN^[[0m /home/runner/work/formal-ai/formal-ai/src/task_decomposition.rs:290:37: macro expansion failed for 'format'
  Line 4042: run-31884932348.log	7627	51c4f0b0ca17	CodeQL (rust) UNKNOWN STEP 2026-08-15T12:35:32.7801724Z [2026-08-15 12:35:32] [build-stdout] [2026-08-15 12:35:32] [build-stdout] ^[[33m WARN^[[0m /home/runner/work/formal-ai/formal-ai/src/task_decomposition.rs:321:26: macro expansion failed for 'format'
  Line 4043: run-31884932348.log	7628	43d48ec5b910	CodeQL (rust) UNKNOWN STEP 2026-08-15T12:35:32.7808006Z [2026-08-15 12:35:32] [build-stdout] [2026-08-15 12:35:32] [build-stdout] ^[[33m WARN^[[0m /home/runner/work/formal-ai/formal-ai/src/task_decomposition.rs:395:9: macro expansion failed for 'format'
  Line 4044: run-31884932348.log	7629	69b42e96815e	CodeQL (rust) UNKNOWN STEP 2026-08-15T12:35:32.7814791Z [2026-08-15 12:35:32] [build-stdout] [2026-08-15 12:35:32] [build-stdout] ^[[33m WARN^[[0m /home/runner/work/formal-ai/formal-ai/src/task_decomposition.rs:464:10: macro expansion failed for 'format'
  Line 4045: run-31884932348.log	7630	e4d9ae7c24d4	CodeQL (rust) UNKNOWN STEP 2026-08-15T12:35:32.7818679Z [2026-08-15 12:35:32] [build-stdout] [2026-08-15 12:35:32] [build-stdout] ^[[33m WARN^[[0m /home/runner/work/formal-ai/formal-ai/src/task_decomposition.rs:485:22: macro expansion failed for 'matches'
  Line 4046: run-31884932348.log	7631	76062a744322	CodeQL (rust) UNKNOWN STEP 2026-08-15T12:35:32.7822901Z [2026-08-15 12:35:32] [build-stdout] [2026-08-15 12:35:32] [build-stdout] ^[[33m WARN^[[0m /home/runner/work/formal-ai/formal-ai/src/task_decomposition.rs:513:22: macro expansion failed for 'vec'
  Line 4047: run-31884932348.log	7632	59db5203e503	CodeQL (rust) UNKNOWN STEP 2026-08-15T12:35:32.7828098Z [2026-08-15 12:35:32] [build-stdout] [2026-08-15 12:35:32] [build-stdout] ^[[33m WARN^[[0m /home/runner/work/formal-ai/formal-ai/src/task_decomposition.rs:536:16: macro expansion failed for 'vec'
  Line 4048: run-31884932348.log	7633	dee3620ef948	CodeQL (rust) UNKNOWN STEP 2026-08-15T12:35:32.7832364Z [2026-08-15 12:35:32] [build-stdout] [2026-08-15 12:35:32] [build-stdout] ^[[33m WARN^[[0m /home/runner/work/formal-ai/formal-ai/src/task_decomposition.rs:575:13: macro expansion failed for 'format'
  Line 4049: run-31884932348.log	7634	0af30ba18401	CodeQL (rust) UNKNOWN STEP 2026-08-15T12:35:32.7838150Z [2026-08-15 12:35:32] [build-stdout] [2026-08-15 12:35:32] [build-stdout] ^[[33m WARN^[[0m /home/runner/work/formal-ai/formal-ai/src/task_decomposition.rs:625:5: macro expansion failed for 'format'

(Results are truncated. Consider using a more specific path or pattern.)
```
