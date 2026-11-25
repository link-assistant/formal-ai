# Cluster c4: Rust test fixes (issue #1637, PR #1638)

Source: `clusters/c4.txt`, CI run 45549993600 (`ci-logs/cicd.clean.log`, `panics.txt`).
No cargo build/check/test/clippy was run (memory constraint). Fixes were verified by
reading the code, `rustfmt --edition 2024 --check` on touched `.rs` files, and small
node probes. Seed edits were mirrored with `node scripts/mirror-package-data.mjs --write`
(then `--check`: all byte-equal) and `scripts/sync-seed.sh`.

## Source target (`rust/tests/source`, a frozen copy of `rust/src`)

### coding::blueprint::tests::selects_python_personal_budget_report_blueprint
- Root cause: 0f4a76eac ("Generate composite programs from caller scenarios") replaced the
  hard-coded `Path('budget_report.md')` with an argparse `output` path
  (`python budget_report.py <scenario.json> <report.md>`).
- Classification: stale test.
- Fix: the test now asserts the run command takes `<scenario.json> <report.md>`, that the
  program declares the `output` argument, and that it writes the report to `args.output`.
- Files: `rust/tests/source/source_tests/coding/blueprint/tests.rs`.
- Confidence: high.

### coding::catalog::lino_parity::lino_seed_task_rows_mirror_every_catalog_task
- Root cause: 9b1ab09ca ("Bind coding catalog operands and derive requested program output")
  made the output of count_to_three, fizzbuzz, factorial, reverse_string, sum_to_ten and
  fibonacci caller-derived (`output: ""`) and removed their seed `output` lines.
- Classification: stale test.
- Fix: a seed `output` row is expected only when the task fixes a non-empty output:
  `(!runtime_input && !task.output.is_empty()).then_some(task.output)`.
- Files: `rust/tests/source/source_tests/coding/catalog/mod/lino_parity.rs`.
- Confidence: high.

### coding::catalog::lino_parity::lino_seed_mirrors_every_catalog_template
- Root cause: 9b1ab09ca added the php/reverse_string template as a backtick value containing
  `implode("", ...)`. The Rust seed parser's `unescape_value` (used for backtick values)
  collapses a doubled `""` to `"`, so Rust read broken PHP: `implode(", ...`. The JS parser
  does not collapse it.
- Classification: product (seed data) regression.
- Fix: write the quotes as `\"\"` inside the backticks. Both parsers decode `\"` to `"`, so
  both now read `implode("", ...)`. `rust/tests/web/issue-0921-worker-coding-catalog.test.mjs`
  shows the JS template comparison passing (13/14; the one failure is a 5 s
  `spawnSync` timeout on the #1613 execution test, `status null`, not related).
- Files: `data/seed/hello-world-programs.lino`, plus its mirrors
  `rust/embedded/data/seed/hello-world-programs.lino` and `js/seed/hello-world-programs.lino`
  (the js/seed copy is gitignored).
- Confidence: high.
- Not done: `unescape_value` doubling `"` inside backtick strings is a latent parser bug.
  Making backtick values use backtick doubling would be the root fix, but it is a shared
  parser change and was left alone.

### coding::catalog::seed_alias_coverage::every_task_alias_meaning_names_a_catalog_slug
- Root cause: 9b1ab09ca added `program-reverse-operand-introducer` with
  `role program_task_alias`. It is not a `program_task_<slug>` meaning; it is only read by
  slug (`catalog_parameters.rs:158`, worker `formal_ai_worker_catalog_parameters.js:67`).
- Classification: seed data regression (wrong role).
- Fix: the role is now the meaning's own slug, following the `line_range_connector` precedent.
  `roles.lino` was regenerated with `python3 scripts/generate-role-registry.py`, and
  `--check` passes (673 roles).
- Side effect: the `mentions_role(program_task_alias)` checks in `obligations.rs:775` and
  worker `program_requests.js:235` no longer see the introducer's surfaces. All of them except
  hi "शब्द उलटें" are also surfaces of `program_task_reverse_string`.
- Files: `data/seed/meanings-coding-catalog.lino`, `data/seed/roles.lino`, and the mirrors
  `rust/embedded/data/seed/meanings-coding-catalog.lino`, `js/seed/*`.
- Confidence: high.

### coding::catalog::seed_alias_coverage::every_language_alias_meaning_names_a_catalog_slug
- Root cause: 10a6f2855 / 289c28b88 added bash, haskell and lua to
  `rust/src/coding/catalog/languages.rs` and the seed. The frozen source copy was not
  updated, so the seed's `program_language_bash` named a slug missing from the copy.
- Classification: stale test copy. Same precedent as e5da0dc1f ("source crate knows Swift and R").
- Fix: added Bash, Haskell and Lua `ProgramLanguage` rows to the copy, using src's
  commands/save_as and the `toolchains.lino` setup hints.
- Knock-on edits that follow from those rows:
  - In `cst/tests.rs`, bash and haskell are declared gaps (no grammar in
    `program-cst-grammars.lino`), and lua leaves `HELD_OUT_GRAMMARS` because it is now a
    catalog language with a grammar.
  - In `numeric_list/tests.rs`, bash, haskell and lua join swift and r as languages with
    no list idioms in `coding-idioms.lino`.
- Files:
  - `rust/tests/source/coding/catalog/languages.rs`
  - `rust/tests/source/source_tests/coding/cst/tests.rs`
  - `rust/tests/source/source_tests/solver_handlers/numeric_list/tests.rs`
- Confidence: medium-high. Each `PROGRAM_LANGUAGES` consumer in the source target was
  checked by reading, not by compiling.

### repository_workspace::tests::module_files_are_where_the_convention_says
- Root cause: dd89c1ecb ("separate solver configuration and command admission") added
  `rust/src/repository_workspace/command_allowlist.rs` (139 lines).
- Classification: stale test.
- Fix: the new file is declared in `DECLARED`, with a doc comment naming the commit.
- Files: `rust/tests/source/source_tests/repository_workspace/tests.rs`.
- Confidence: high.

## Seed / reference closure

### seed::reference_closure::meaning_definition_references_resolve_to_defined_meanings
- Already fixed by 0f0266dc1 (`urge -> concept`, `requested-text-sentiment -> message`;
  both targets exist). No further change.
- Confidence: high.

## Unit target

### coding_discovery::multilingual::count_to_one_hundred_requests_derive_the_full_inclusive_output
- Root cause: 9b1ab09ca gave `count_to_three` generic "count to" aliases, so the gap name is
  now `count_to_three` rather than `main` (the Hindi paraphrase still gets `main`).
- Classification: stale test.
- Fix: the test accepts `main` or `count_to_three`. It adds checks for the Program shape and
  empty parameters, and keeps the composed `range(1, 100 + 1)` checks.
- Files: `rust/tests/unit/coding_discovery/multilingual.rs`.
- Confidence: medium-high.

### coding_discovery::task_spec::conversational_program_without_a_named_callable_uses_the_main_entry_point
- Root cause: 9b1ab09ca (as above).
- Classification: stale test.
- Fix: the test uses a prompt that matches no catalog alias ("print the squares of the numbers
  up to 10"; checked with `experiments/c4_task_alias_probe.mjs`). Assertions are unchanged.
- Files: `rust/tests/unit/coding_discovery/task_spec.rs`.
- Confidence: medium-high.

### issue_1021_behaviour_range::{a_framework_named_coding_request_keeps_the_framework, asking_for_code_is_a_coding_request_whatever_the_asking_verb}
- Root cause: 9adbb43f5 ("Bind script operands …") changed these tests to expect
  `program_parameter:language <lang>` evidence on the `write_program_skill_gap` dead end.
  Only the JS worker logged it; the Rust `SelectedRule::UnsupportedWriteProgram` path did not.
- Classification: product parity gap.
- Fix: the Rust dead end now logs `program_parameter:language` and `program_parameter:task`.
  A missing value is logged as `program_skill_gap::MISSING_PARAMETER`.
- Files: `rust/src/solver.rs` (shared file, a minimal addition in the dead-end block).
- Confidence: medium.

### issue_1021_closed_circle::closed_circle_session_replays
- Root cause: 9adbb43f5 intentionally moved the #723 Laravel prompts from
  `write_script_laravel` to `write_program_skill_gap` (a request without a task asks for the
  task), as the behaviour_range tests above now pin. The capture still had the old intent.
- Classification: stale capture.
- Fix: the two `intent` values in the capture were hand-edited. The edited file is
  byte-identical (after trim) to the fresh JSON printed in the CI panic, so it equals what
  `FORMAL_AI_UPDATE_CLOSED_CIRCLE=1` would write.
- Files: `docs/case-studies/issue-1021/closed-circle-run/session.json`.
- Confidence: high.

### issue_1588_intent_spans::document_intent_and_bound_quantities_match_worker_fixtures
- Root cause: 91a64e8d3 logged `calendar:result_date` but never added a rendering arm in
  `event_log.rs`. The link rendered as `calendar:result_date:<event id>` instead of the date.
- Classification: product bug.
- Fix: added a `"calendar:result_date" => format!("calendar:result_date:{}", payload)` arm.
- Files: `rust/src/event_log.rs`.
- Confidence: high.

### issue_901_triz_solver::a_russian_contradiction_prompt_gets_the_localized_worksheet
- Root cause: 1ec3ca802 made `triz_demands` start reading right after the cue match. The stem
  cue "противоречи" left "е: " of "противоречие:" in the first demand.
- Classification: product regression.
- Fix: after a cue that ends mid-word, the reader skips to the end of the word (scripts
  without spaces are left alone). Checked with `experiments/c4_triz_stem_probe.mjs`.
- Files: `rust/src/triz_solver.rs`.
- Confidence: medium-high.
- Proposed, not done: the same fix in `js/worker/formal_ai_worker_triz.js` and the ts twin,
  whose loose test regex hides the bug.

### issue_918_handler_rules_batch::brainstormed_project_ideas_use_the_requested_topic_and_count
### issue_918_handler_rules_batch::the_algorithm_rules_compose_the_requested_language
- Root cause: `question_necessity::enforce_questions` strips any question whose wording it
  does not recognise. Two replies lost their opening question:
  - 847753b12 reduced the brainstorming refusal to a bare question, which was stripped,
    leaving an empty answer.
  - 99a1953c3's `algorithm-operation-required` reply lost its "What should the algorithm do?".
- Classification: product regression.
- Fix: a new class record, `missing-choice-requirement-question` (en/ru/hi/zh/es), lists both
  question forms so they are kept.
- Files: `data/seed/question-necessity-text-predicates.lino`, plus its mirrors in
  `rust/embedded/data/seed/` and `js/seed/`.
- Confidence: medium-high.

### web_engine_core::issue_1177_code_task_handlers::engine_answers_shell_compose_request
- Root cause: bed7ae40e added a separate `"arithmetic"` path in `try_contextual_override`.
  That path skipped the `shell_compose_claims` guard and `record_validation`, so a `find`
  request was answered as failed arithmetic.
- Classification: product regression.
- Fix: both paths share `arithmetic_with_online(prompt, log, online)`, which keeps the guards.
- Files: `rust/src/solver_dispatch.rs` (shared router; minimal).
- Confidence: medium-high.

### solver::issue_1138_self_use_concept_lookup::{a_spanish_prompt_is_not_reported_as_an_unsupported_language, an_unresolved_word_is_looked_up_rather_than_answered_with_the_provider_description}
- Root cause: c0e480a36 (#1582) turned source reads on by default. The pinned
  `offline_cache_miss` statuses were recorded with the network denied; CI reached the network
  and got `unreachable`.
- Classification: stale test (depends on the environment).
- Fix: the tests replay in a child with `FORMAL_AI_NETWORK=denied` and an empty private cache,
  using the new helper `offline_replay::in_network_denied_child`. Expected strings are
  unchanged.
- Files: `rust/tests/unit/solver/issue_1138_self_use_concept_lookup.rs`,
  `rust/tests/unit/offline_replay.rs`.
- Confidence: medium-high.

### solver::issue_1175_routing_probes::the_native_solver_answers_every_probe_as_measured
- Root cause: 60c120811 made elided "как …" leads guess a verb, so «Как ты думаешь, есть ли
  свобода воли?» (probe p207) went to `procedural_how_to`.
- Classification: product regression.
- Fix: `extract_elided_procedural_how_to_task` declines when the `opinion_question` handler
  matches.
- Files: `rust/src/solver_handler_how.rs`.
- Confidence: medium-high.
- Proposed, not done: the same guard in the JS/TS worker twins.

### solver::issue_938_meta_algorithm_builder::{all_coding_surfaces_execute_one_trace_shape, shared_program_synthesis_trace_is_multilingual}
- Root cause: c30b54f15 added `try_bound_publisher_recurrence`. It wrote the 7-stage builder
  trace even when recurrence discovery found nothing, and normal synthesis then wrote it
  again (14 stages).
- Classification: product regression.
- Fix: in recurrence-only mode the trace is written only after discovery succeeds.
- Files: `rust/src/solver_handlers/program_synthesis.rs`.
- Confidence: medium-high.

### unresolved_request_research::offline_mode_preserves_the_explicit_no_network_boundary
- Root cause: be175ba74 (#1580) routes seeded action prompts to `web_search` even offline,
  answering from the cache only.
- Classification: stale test.
- Fix: the test asserts intent `web_search`, the `query_kind:unknown_reasoning_fallback`
  evidence, and that no `web_search:executed` link appears (the no-network boundary).
- Files: `rust/tests/unit/unresolved_request_research.rs`.
- Confidence: high.

### issue_823_recursive_learning::the_live_solver_recalls_an_approved_lesson_instead_of_rederiving_it
- No panic text exists (the shard log is truncated).
- Root cause: 289c28b88 removed the built-in approved-lessons seed. The ledger now needs
  `FORMAL_AI_APPROVED_LESSONS`, and it is a process-wide `OnceLock`.
- Classification: stale test.
- Fix: the test replays in a child via the new helper
  `offline_replay::in_child_with_approved_lessons`, using
  `rust/tests/fixtures/issue-558/approved-lessons.lino`.
- Files: `rust/tests/unit/issue_823_recursive_learning.rs`, `rust/tests/unit/offline_replay.rs`.
- Confidence: medium-high.

### seed::issue_1173_route3_probes::a_program_that_prints_a_greeting_binds_the_output_operand
- No panic text exists.
- Most plausible cause: since c0e480a36, other tests in the shard fill the shared source cache.
  This `offline: true` test then rendered a "rediscovered from doc.rust-lang.org" answer
  instead of the pinned offline text.
- Classification: stale test (depends on the environment).
- Fix: the test runs via `offline_replay::in_child` with a private empty cache.
- Files: `rust/tests/unit/seed/issue_1173_route3_probes.rs`.
- Confidence: medium.

## Shared-file edits
- `rust/src/solver.rs`: the dead end logs `program_parameter:*`.
- `rust/src/solver_dispatch.rs`: the arithmetic override reuses the guarded handler.
- `rust/tests/unit/offline_replay.rs`: `replay_in_child` takes env pairs; two new helpers were
  added, and the existing helpers behave as before.

## Follow-ups not done
- JS/TS twins for the TRIZ stem fix and the opinion-question guard.
- The frozen copies `rust/tests/source/solver_handler_how.rs` and
  `rust/tests/source/solver_handlers/program_synthesis.rs` were not synced; no source test
  needs them.
- `an_honest_refusal_names_every_source_it_consulted` (issue_1138, not in c4) probably needs the
  same network-denied wrapper.
- `rust/tests/unit/offline_instruction_research.rs` is not declared in `rust/tests/unit/mod.rs`.
