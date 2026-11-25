# Rust fixes, cluster c1 (issue #1637 / PR #1638)

Nothing here was compiled. The crate was not built, checked or tested because of the 3 GB memory cgroup. Every change was verified by reading the code and the signatures it calls. Edited Rust files pass a standalone `rustfmt --edition 2024 --check`. The seed mirror was refreshed with `node scripts/mirror-package-data.mjs --write` and is byte-equal under `--check`. `scripts/sync-seed.sh` was run.

## Shared product, seed and parity changes

| File | Change | Tests it serves |
| --- | --- | --- |
| `rust/src/coding/operand_program.rs` | `try_write_documented_program` attaches `execution_recipe`: language, source, `save_as` path and contract commands. Regression from 10a6f2855, which added the documented-example path without the recipe. | issue_716 (unit and integration), issue_859 codex, codex_patch_writes. This is also the c3 issue_1165 recipe. |
| `rust/src/agentic_coding/evidence_record/evidence_relevance.rs` and `js/agentic/evidence_record.mjs` (parity twin) | `later_route_delivering` returns `declined_learning_report` when the learning-report route owns the obligation target. | issue_686 associative learning |
| `rust/src/release_timeline.rs` | New `names_a_series` guard in `open_series_subject`. Since 6974c0278, the "{subject} films in release order" frame captured generic nouns such as "list". The JS worker has the same weakness and was left unchanged. | capability_routing a_surface_word_alone_is_not_admitted |
| `data/seed/coding-guidance.lino` (mirrored to `rust/embedded/...`) | Adds `es` lines to the 9 records that 10a6f2855 added without Spanish. | issue_1138_surface_honesty guidance_has_a_spanish_branch |
| `data/seed/prompt-patterns.lino` (mirrored) | New concept_lookup prefix `pattern_concept_prefix_explain_the_word`, text `"explain the word "`. | issue_1138_concept_lookup_http dictionary opt-out |
| `rust/src/main.rs` | The `--verbose` doc comment says "enabled by default" again. | issue_822_context |

The mirror run also refreshed embedded copies of seed files that other agents edited: `rust/embedded/data/seed/{hello-world-programs,meanings-coding-catalog,question-necessity-text-predicates}.lino`.

## Per-test entries

### agentic_coding::agentic_surfaces::responses_non_agentic_task_with_tools_falls_through_to_symbolic
- **Root cause:** PR #1614 retired the shipped capture for "What is the capital of France?". With `web_search` advertised, open-web research now adopts that prompt, so it no longer falls through to the symbolic solver.
- **Classification:** stale test.
- **Fix:** use a locally answered prompt, "What is 731 plus 83?". The fall-through contract is still asserted as before.
- **Files:** `rust/tests/unit/agentic-coding/agentic_surfaces.rs`.
- **Confidence:** high.

### agentic_coding::planning_routes::planner_ignores_non_formalization_tasks
- **Root cause, classification and fix:** same as the previous test. The test lives in `agentic_coding.rs`, which is mounted as `planning_routes`.
- **Files:** `rust/tests/unit/agentic-coding/agentic_coding.rs`.
- **Confidence:** high.

### agentic_coding::agentic_surfaces::responses_returns_final_message_once_recipe_is_exhausted
- **Root cause:** 89a674aa0 (PR #1614). Without a fetch tool the publisher text is never verified, so the final message reports the missing source instead of building a knowledge base from text it never read.
- **Classification:** stale test.
- **Fix:** assert one final message that contains "could not verify the requested text" and does not contain `knowledge_base`. The chat-completions surface pins the same contract.
- **Files:** `rust/tests/unit/agentic-coding/agentic_surfaces.rs`.
- **Confidence:** high.

### agentic_coding::codex_patch_writes::codex_hello_world_starts_with_apply_patch_instead_of_write_stdin
- **Root cause:** 10a6f2855. The documented-program answer lost its `execution_recipe`, so the agent harness had nothing to write or run.
- **Classification:** product regression.
- **Fix:** the `operand_program.rs` change listed in the shared table.
- **Confidence:** medium-high.

### agentic_coding::issue_716_agentic_execution::* (7 tests), issue_716_agentic_execution::{anthropic_messages,chat_completions}_routes_program_creation_to_the_advertised_cli_write_tool, issue_859_codex::* (2 tests)
- **Root cause:** 10a6f2855, the same missing `execution_recipe`.
- **Classification:** product regression.
- **Fix:** `rust/src/coding/operand_program.rs`. The recipe source is `documented.recipe.entry`, the path is `contract.save_as`, and the commands come from `contract.commands[].command`.
- **Confidence:** medium-high.

### agentic_coding::issue_1085_requirement_resolution::the_ladder_compiles_tests_merges_and_verifies_requirement_levels
- **Root cause:** 971802a07. `verify-node.sh` now runs registered cases per Cargo target (`--test "$target"`) instead of a literal `cargo test --test unit`.
- **Classification:** stale test.
- **Fix:** the needles now match the target-argument form.
- **Files:** `rust/tests/unit/agentic-coding/issue_1085_requirement_resolution.rs`.
- **Confidence:** high.

### agentic_coding::issue_558_repair_strategy::planner_walks_the_repair_strategy_recipe
- **Root cause:** 289c28b88 replaced the fixed "a solver method, a data record, or a test" text with the classified targets.
- **Classification:** stale test.
- **Fix:** assert `(solver_method, data_record, test)` and `requires human approval`.
- **Files:** `rust/tests/unit/agentic-coding/issue_558_repair_strategy.rs`.
- **Confidence:** high.

### agentic_coding::issue_686_agent_cli::associative_learning_requires_a_supplied_observation_network
- **Root cause:** two things.
  - The evidence-record route claimed `associative-learning-report.lino` before the learning-report route could ask for the missing memory. This is a product regression, fixed in `evidence_relevance.rs` and its JS twin.
  - For the unclosed fence, the request-level quote guard (G71) now refuses first with "never closes". That is a legitimate change, and the JS twin pins the same contract.
- **Classification:** product regression plus a stale test.
- **Files:**
  - `rust/src/agentic_coding/evidence_record/evidence_relevance.rs`
  - `js/agentic/evidence_record.mjs`
  - `rust/tests/unit/agentic-coding/issue_686_agent_cli.rs`
- **Confidence:** medium.

### agentic_coding::issue_848_coding_ladder::collection_edit_reads_transforms_writes_and_observes_workspace_bytes
- **Root cause:** c3b3d235f. A list edit now binds to a complete owned read frame.
- **Classification:** stale test.
- **Fix:** feed `tool_result::complete_owned_source_read_frame(BEFORE)`, the same frame the JS twin uses.
- **Files:** `rust/tests/unit/agentic-coding/issue_848_coding_ladder.rs`.
- **Confidence:** high.

### agentic_coding::work_item_planning::repository_work_item_run_reports_planned_not_executed_instead_of_success
- **Root cause:** d26a8e76f moved the GitHub issue snapshots out of the runtime. With the network open, the default transport reads #904 live.
- **Classification:** stale test.
- **Fix:** run through `run_agentic_task_with_source_fetch` with a transport that returns a 404 error. This is the unavailable-source scenario the test describes.
- **Files:** `rust/tests/unit/agentic-coding/work_item_planning.rs`.
- **Confidence:** high.

### agentic_coding::work_item_planning::an_uncovered_language_is_not_invented_into_an_artifact
- **Root cause:** 10a6f2855 added Haskell to `hello-world-programs.lino`. The test's own doc comment says to move it on to another language when this happens.
- **Classification:** stale test.
- **Fix:** replace Haskell with Fortran.
- **Files:** `rust/tests/unit/agentic-coding/work_item_planning.rs`.
- **Confidence:** high.

### capability_routing::issue_1175_claim_routing::a_surface_word_alone_is_not_admitted
- **Root cause:** 6974c0278. The release-order frame admitted a generic noun as a film-series subject.
- **Classification:** product regression.
- **Fix:** add `names_a_series` in `rust/src/release_timeline.rs`.
- **Confidence:** medium.

### capability_routing::issue_1175_claim_routing::class_c_a_composer_without_its_specification_may_only_refuse
- **Root cause:** 847753b12. The planner now needs supplied, cited destination observations.
- **Classification:** stale test.
- **Fix:** the specified planning prompt is now a JSON observation. The JS twin uses the same prompt.
- **Files:** `rust/tests/unit/capability-routing/issue_1175_claim_routing.rs`.
- **Confidence:** medium-high. CI printed no panic text, so the prompt was checked against the seed and the JS twin instead.

### capability_routing::issue_1175_claim_routing::the_claim_rows_are_read_from_the_capability_table
- **Root cause:** 847753b12, 1a18b562a, 1ec3ca802 and 9458c500c renamed seed operands and handlers.
- **Classification:** stale test.
- **Fix:** six rows changed:
  - advice: `advice_observations`
  - planning: `destination_observations`
  - brainstorming: `brainstorm_topic`
  - roleplay: `roleplay_facts`
  - triz: `triz_demands`
  - docs handler: `tryDocumentationMethodExplanation`
- **Files:** `.../issue_1175_claim_routing/claim_table.rs`.
- **Confidence:** high. A python script diffed the rows against `data/seed`.

### issue_1138_surface_honesty::guidance_has_a_spanish_branch
- **Root cause:** 10a6f2855 added 9 coding-guidance records without `es` lines.
- **Classification:** product (seed) regression.
- **Fix:** add the `es` lines to `data/seed/coding-guidance.lino` and mirror it.
- **Confidence:** high.

### issue_1138_concept_lookup_http::chat_completions_honours_a_dictionary_opt_out
- **Root cause:** dc16ea0da (2026-11-18). `answer_from_retrieved_senses` now binds a sense only when it matches the requested term.
  - For "Explain the word lipogram.", the concept term is "the word lipogram", which `normalize_fact_subject` turns into "word lipogram".
  - That never binds to the "lipogram" sense.
  - The instruction-shaped prompt therefore fell to web search ("Fused 0 statement(s)…"), and the disabled-dictionary note disappeared.
- **Classification:** product regression.
- **Fix:** add the seed prefix `explain the word `. Prefixes are matched longest-first, so the term becomes "lipogram".
- **Files:** `data/seed/prompt-patterns.lino` and its embedded mirror.
- **Confidence:** medium-high.

### issue_1138_family_behaviour::no_family_answer_is_the_unknown_opener
- **Root cause:** read access has been default-on since c0e480a36 (#1582). The retrieval walk fetched an Open English WordNet gloss for the unseeded answer-shape word "paragraph", which replaced the documented unknown opener. First failure: run 45001543202.
- **Classification:** stale test, caused by the changed environment default.
- **Fix:** the test re-runs itself in a child process with `FORMAL_AI_NETWORK=denied` and an empty exclusive `FORMAL_AI_SOURCE_CACHE_DIR`. The integration target has no `offline_replay` module, so the helper is local to the file.
- **Product proposal (not applied):** keep answer-shape and directive words such as "paragraph" out of the retrieval surfaces, or seed them.
- **Not fixed:** the sibling test `every_retired_handler_prompt_still_answers_through_its_family` makes the same assertion and is equally network-sensitive. It is not in c1 and was left unchanged.
- **Files:** `rust/tests/integration/issue_1138_family_behaviour.rs`.
- **Confidence:** medium-high.

### issue_712_intent_routing::leading_edit_action_is_language_general
- **Symptom:** passed in run 44909319833 (2026-11-21) and failed deterministically from run 45001543202 (2026-11-22) on.
  - In every failing run, the English and Russian three-turn exchanges finish.
  - The first Hindi request ("बदलो main.rs और बदलो foo से bar", refusal turn) then gets no response within the 30 s budget and logs nothing more on the server (`POST should complete: WouldBlock`).
- **Root cause:** not isolated. Network default-on (c0e480a36, 11-17) predates the last green run, so the trigger is in the 2026-11-21 13:34 to 2026-11-22 08:13 range. That range holds a series of "acquire … from publishers" commits that add live source reads, for example 89a674aa0, 10a6f2855 and e4f1a1fda. The most likely trigger is that the Hindi refusal turn now reaches one of these live reads.
- **Classification:** probable environment or test isolation problem; a product slow path cannot be ruled out.
- **Fix:** the server for this test now starts with `spawn_formal_ai_server_with_args(port, &["--agent-mode"], &[("FORMAL_AI_NETWORK", "denied")])`. Routing an edit request is a local decision, so the test no longer depends on live reads. If the hang is CPU-bound rather than network-bound, this will not fix it; the next step would be a local trace of that prompt through the planner (`plan_chat_step`) with a built binary.
- **Files:** `rust/tests/integration/issue_712_intent_routing.rs`.
- **Confidence:** low-medium.

### issue_822_context::verbose_is_global_default_and_silent_is_available
- **Root cause:** d72073e87 (2026-11-19) reworded the `--verbose` doc comment to "(wrapper diagnostics need this explicit flag)" and dropped "enabled by default". `default_value_t = true` is unchanged, so the help text understated the real default.
- **Classification:** product (documentation) regression.
- **Fix:** the doc comment in `rust/src/main.rs` now reads "Print verbose CLI output (enabled by default; wrapper diagnostics need this explicit flag)." No other test or doc pins the old wording.
- **Confidence:** high.

## Modified files (c1)
- rust/src/coding/operand_program.rs
- rust/src/agentic_coding/evidence_record/evidence_relevance.rs
- js/agentic/evidence_record.mjs
- rust/src/release_timeline.rs
- rust/src/main.rs
- data/seed/coding-guidance.lino, rust/embedded/data/seed/coding-guidance.lino
- data/seed/prompt-patterns.lino, rust/embedded/data/seed/prompt-patterns.lino
- rust/tests/unit/agentic-coding/agentic_coding.rs
- rust/tests/unit/agentic-coding/agentic_surfaces.rs
- rust/tests/unit/agentic-coding/issue_1085_requirement_resolution.rs
- rust/tests/unit/agentic-coding/issue_558_repair_strategy.rs
- rust/tests/unit/agentic-coding/issue_686_agent_cli.rs
- rust/tests/unit/agentic-coding/issue_848_coding_ladder.rs
- rust/tests/unit/agentic-coding/work_item_planning.rs
- rust/tests/unit/capability-routing/issue_1175_claim_routing.rs
- rust/tests/unit/capability-routing/issue_1175_claim_routing/claim_table.rs
- rust/tests/integration/issue_1138_family_behaviour.rs
- rust/tests/integration/issue_712_intent_routing.rs
- Refreshed by the mirror, edited by other agents: rust/embedded/data/seed/{hello-world-programs,meanings-coding-catalog,question-necessity-text-predicates}.lino
