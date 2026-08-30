node_path=1.1.1.1.2

The `grep` command completed. Output:

```text
Found 100 matches
/tmp/tmp.DpPaeG2eEF/experiments/issue-847-self-hosting-evidence/run.sh:
  Line 4: # one recursion in src/task_decomposition.rs) — is hand-authored maintenance,
  Line 35: #      module — including src/task_decomposition.rs and
  Line 36: #      src/solver_handlers/task_decomposition.rs, the modules this PR adds — to

/tmp/tmp.DpPaeG2eEF/examples/issue_1066_decomposition_probe.rs:
  Line 5:     let decomposition = formal_ai::task_decomposition::decompose_task(&task, 4);
  Line 14:         formal_ai::task_decomposition::split_once_checkable(&task)

/tmp/tmp.DpPaeG2eEF/examples/dump_task_decomposition.rs:
  Line 3: //! Usage: `cargo run --example dump_task_decomposition -- "<task>" [max_depth]`
  Line 5: use formal_ai::task_decomposition::{decompose_task, is_checkable};

/tmp/tmp.DpPaeG2eEF/CHANGELOG.md:
  Line 65: - Failure-driven splitting: `TaskExecutor` gained a `split` hook, so a failed task can be shrunk from its own failure instead of from a plan made before any evidence existed. `formal_ai::task_decomposition::SplittingExecutor` answers that hook with the repository's own `decompose_task`, one level per split, and records every split with the failure that justified it. The controller refuses a child that repeats its parent, bounds splitting with `DEFAULT_SPLIT_DEPTH_BOUND`, and `solve_recursively_within` lets a caller pick another bound (zero reproduces the previous plan-driven protocol exactly).

/tmp/tmp.DpPaeG2eEF/examples/issue_1066_checkable_probe.rs:
  Line 8:             formal_ai::task_decomposition::is_checkable(&segment)

/tmp/tmp.DpPaeG2eEF/scripts/tests-as-docs-allowlist.txt:
  Line 106: tests/unit/issue_847_task_decomposition.rs	a_composite_task_is_reported_as_not_atomic
  Line 107: tests/unit/issue_847_task_decomposition.rs	asking_whether_a_task_is_atomic_is_answered_standalone
  Line 108: tests/unit/issue_847_task_decomposition.rs	atomicity_is_answerable_in_every_supported_language
  Line 109: tests/unit/issue_847_task_decomposition.rs	decomposition_is_recognised_in_every_supported_language
  Line 110: tests/unit/issue_847_task_decomposition.rs	splitting_a_coding_task_into_subtasks_is_answered_not_refused
  Line 390: tests/unit/specification/task_decomposition.rs	atomicity_is_recognised_in_every_supported_language
  Line 391: tests/unit/specification/task_decomposition.rs	the_configured_depth_bound_is_reported_in_the_answer

/tmp/tmp.DpPaeG2eEF/src/solver_dispatch.rs:
  Line 37:     try_summarization_request, try_task_decomposition_with_depth, try_text_manipulation,
  Line 140:     "task_decomposition",
  Line 217:         "task_decomposition" => try_task_decomposition_with_depth(

/tmp/tmp.DpPaeG2eEF/src/intent_formalization/prompt_relevants.rs:
  Line 51:             "handler:task_decomposition",
  Line 52:             crate::solver_handlers::looks_like_task_decomposition(normalized),

/tmp/tmp.DpPaeG2eEF/src/recursive_execution.rs:
  Line 15: //! honest move. [`crate::task_decomposition::SplittingExecutor`] wires the

/tmp/tmp.DpPaeG2eEF/src/engine.rs:
  Line 128:     /// so ([`crate::task_decomposition::Decomposition::unenumerable_reason`]);

/tmp/tmp.DpPaeG2eEF/experiments/issue_1066_ladder_offline/falsify-node-capabilities.sh:
  Line 51:   "src/task_decomposition.rs|    pub fn unenumerable_reason(&self) -> Option<AtomicityReason> {|if true { return None; }|issue_1066_hollow_answers::an_answer_that_announces_sub_tasks_never_lists_none"
  Line 52:   "src/task_decomposition/stated_task.rs|pub fn without_sentence_end(task: &str) -> &str {|if true { return task; }|issue_1066_hollow_answers::a_listed_sub_task_keeps_the_text_that_says_what_to_do"
  Line 55:   "src/task_decomposition/stated_task.rs|fn after_introducing_colon(prompt: &str, asks: &dyn Fn(&str) -> bool) -> Option<String> {|if true { let colon = prompt.rfind(INTRODUCING_COLON)?; let tail = prompt[colon..].chars().skip(1).collect::<String>().trim().to_owned(); return (!tail.is_empty()).then_some(tail); }|issue_1066_hollow_answers::a_colon_in_a_later_sentence_does_not_become_the_task"
  Line 56:   "src/task_decomposition/stated_task.rs|fn asking_blocks(prompt: &str, asks: &dyn Fn(&str) -> bool) -> String {|if true { return prompt.trim().to_owned(); }|issue_1066_hollow_answers::framing_addressed_to_the_solver_is_not_a_sub_task"

/tmp/tmp.DpPaeG2eEF/experiments/issue_989_self_authoring/run.sh:
  Line 10: issue_989_task_decomposition
  Line 17:   leaf reviewed_task_decomposition author formal_ai'

/tmp/tmp.DpPaeG2eEF/src/solver_handlers/modules.rs:
  Line 41: mod task_decomposition;

/tmp/tmp.DpPaeG2eEF/src/solver_handlers/mod.rs:
  Line 42: pub use task_decomposition::{looks_like_task_decomposition, try_task_decomposition_with_depth};

/tmp/tmp.DpPaeG2eEF/src/solver_handlers/task_decomposition.rs:
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

/tmp/tmp.DpPaeG2eEF/src/agentic_coding/task_structure.rs:
  Line 7: //! [`crate::solver_handlers::task_decomposition`], which has answered the three
  Line 17: //! ([`looks_like_task_decomposition`]), so every phrasing and every language the
  Line 39: use crate::solver_handlers::looks_like_task_decomposition;
  Line 67:     if !looks_like_task_decomposition(&normalize_prompt(task)) {

/tmp/tmp.DpPaeG2eEF/experiments/issue_847_self_authoring/run.sh:
  Line 10: task_decomposition_contract

/tmp/tmp.DpPaeG2eEF/examples/issue_991_incremental_decomposition.rs:
  Line 15: use formal_ai::task_decomposition::SplittingExecutor;

/tmp/tmp.DpPaeG2eEF/src/agentic_coding/stated_request.rs:
  Line 9: //! The rule is the one [`crate::task_decomposition`] already applies when it

/tmp/tmp.DpPaeG2eEF/src/agentic_coding/shell_command.rs:
  Line 251: /// `task_decomposition`". A request that only says what the caller wants to
  Line 331: /// Prose names an identifier by writing it: `task_decomposition`,

/tmp/tmp.DpPaeG2eEF/src/task_decomposition/artifact.rs:
  Line 31:             .find(|node| node.find_child_value("record_type") == "task_decomposition")
  Line 32:             .ok_or_else(|| error("missing_task_decomposition"))?;
  Line 70:         let tree_digest = stable_id("task_decomposition_tree", &canonical_tree);
  Line 88:         let tree_digest = stable_id("task_decomposition_tree", &tree);
  Line 92:                 ("record_type", "task_decomposition".to_owned()),
  Line 194:     stable_id("task_decomposition", &identity)

/tmp/tmp.DpPaeG2eEF/src/task_decomposition/strategy.rs:
  Line 73:         .find(|node| node.name == "task_decomposition_strategies")
  Line 175:         .find(|node| node.name == "task_decomposition_strategies")
  Line 214:         .find(|node| node.name == "task_decomposition_contract")?;

/tmp/tmp.DpPaeG2eEF/src/orchestration/incremental.rs:
  Line 12: //! [`crate::task_decomposition::SplittingExecutor`], so the shrink-on-failure
  Line 43: use crate::task_decomposition::SplittingExecutor;
  Line 83: /// [`crate::task_decomposition::TaskStrategyLedger`] applies to a learned

/tmp/tmp.DpPaeG2eEF/src/orchestration/dispatch.rs:
  Line 9: use crate::task_decomposition::decompose_task;

/tmp/tmp.DpPaeG2eEF/experiments/issue_961_self_authoring/run.sh:
  Line 24: issue_961_task_decomposition
  Line 33:   leaf reviewed_task_decomposition author formal_ai'

/tmp/tmp.DpPaeG2eEF/src/seed/roles/decomposition.rs:
  Line 12: /// are caught. Carried by `task_decomposition_action`; the decomposition
  Line 15: pub const ROLE_TASK_DECOMPOSITION_ACTION: &str = "task_decomposition_action";

/tmp/tmp.DpPaeG2eEF/tests/unit/mod.rs:
  Line 174: mod issue_847_task_decomposition;

/tmp/tmp.DpPaeG2eEF/dev/log/issues/1014/pulls/1015/raw-data/all-diagnostic-candidates.tsv:
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
  Line 4050: run-31884932348.log	7635	a9d9af465d44	CodeQL (rust) UNKNOWN STEP 2026-08-15T12:35:32.7842924Z [2026-08-15 12:35:32] [build-stdout] [2026-08-15 12:35:32] [build-stdout] ^[[33m WARN^[[0m /home/runner/work/formal-ai/formal-ai/src/task_decomposition.rs:667:13: macro expansion failed for 'format'
  Line 4051: run-31884932348.log	7636	2527fde5fd72	CodeQL (rust) UNKNOWN STEP 2026-08-15T12:35:32.7909564Z [2026-08-15 12:35:32] [build-stdout] [2026-08-15 12:35:32] [build-stdout] ^[[33m WARN^[[0m /home/runner/work/formal-ai/formal-ai/src/task_decomposition/artifact.rs:178:31: macro expansion failed for 'format'
  Line 4052: run-31884932348.log	7637	aa133515d724	CodeQL (rust) UNKNOWN STEP 2026-08-15T12:35:32.7964071Z [2026-08-15 12:35:32] [build-stdout] [2026-08-15 12:35:32] [build-stdout] ^[[33m WARN^[[0m /home/runner/work/formal-ai/formal-ai/src/task_decomposition/learning.rs:394:31: macro expansion failed for 'format'
  Line 4053: run-31884932348.log	7638	9ca32ab37eef	CodeQL (rust) UNKNOWN STEP 2026-08-15T12:35:32.7967901Z [2026-08-15 12:35:32] [build-stdout] [2026-08-15 12:35:32] [build-stdout] ^[[33m WARN^[[0m /home/runner/work/formal-ai/formal-ai/src/task_decomposition/learning.rs:400:29: macro expansion failed for 'format'
  Line 4054: run-31884932348.log	7639	5ea303cc0762	CodeQL (rust) UNKNOWN STEP 2026-08-15T12:35:32.8025775Z [2026-08-15 12:35:32] [build-stdout] [2026-08-15 12:35:32] [build-stdout] ^[[33m WARN^[[0m /home/runner/work/formal-ai/formal-ai/src/task_decomposition/strategy.rs:12:5: macro expansion failed for 'include_str'
  Line 4055: run-31884932348.log	7640	c5ad9fd92cbf	CodeQL (rust) UNKNOWN STEP 2026-08-15T12:35:32.8028347Z [2026-08-15 12:35:32] [build-stdout] [2026-08-15 12:35:32] [build-stdout] ^[[33m WARN^[[0m /home/runner/work/formal-ai/formal-ai/src/task_decomposition/strategy.rs:13:33: macro expansion failed for 'include_str'
  Line 4056: run-31884932348.log	7641	deb2cdd59a07	CodeQL (rust) UNKNOWN STEP 2026-08-15T12:35:32.8038041Z [2026-08-15 12:35:32] [build-stdout] [2026-08-15 12:35:32] [build-stdout] ^[[33m WARN^[[0m /home/runner/work/formal-ai/formal-ai/src/task_decomposition/strategy.rs:148:13: macro expansion failed for 'matches'
  Line 4057: run-31884932348.log	7642	8c60059ee551	CodeQL (rust) UNKNOWN STEP 2026-08-15T12:35:32.8046938Z [2026-08-15 12:35:32] [build-stdout] [2026-08-15 12:35:32] [build-stdout] ^[[33m WARN^[[0m /home/runner/work/formal-ai/formal-ai/src/task_decomposition/strategy.rs:246:12: macro expansion failed for 'matches'

(Results are truncated. Consider using a more specific path or pattern.)
```
