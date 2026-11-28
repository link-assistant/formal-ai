# Cluster c2 (`specification`) — Rust test fixes

Verification note: nothing in this cluster was compiled or run. The formal-ai
crate cannot be built in this sandbox (it is OOM-killed). Every fix was checked
by reading the code. Edited Rust files pass `rustfmt --edition 2024 --check`.
Where possible, behaviour was confirmed with small node or python scripts:
- `splitRequest` before and after the fix;
- a simulation of blueprint keyword detection;
- `mirror-package-data.mjs --check`;
- `generate-worker-crate-modules.mjs --check`.

CI is the real verification.

## Summary table

| Test | Class | Root cause | Confidence |
| --- | --- | --- | --- |
| agentic_meta_algorithm::meta_functions_exist_in_named_source, meta_surfaces_expose_the_loop | stale recipe data | a585d4ec7 moved `classify_tool` to `capability_router/tool_classification.rs`; 62c794be7 moved the CLI surface to `src/cli_solver.rs` | high |
| meta_algorithm::meta_recipe_functions_exist_in_named_source | stale recipe data | `answer_markdown` lives in `src/how_to_guide/render.rs` | high |
| concept_lookup_meta_algorithm::concept_lookup_recipe_is_grounded_in_the_live_retrieval_kernel | stale test and recipe | 36c0e3c3f split the surface matching into `concept_lookup/unknown_surfaces.rs` | high |
| budget_search_meta_algorithm::meta_recipe_integration_is_wired_into_the_loop_with_budget_controls | stale test | dd89c1ecb moved the configuration to `src/solver/configuration.rs` | high |
| web_app_sources_cover_every_jsx_module | stale test | 409d72d08 added `js/app/cnl-settings.jsx` | high |
| method_registry::registry_covers_all_dispatch_surfaces | stale test | the contextual override table grew from 14 to 18 entries | high |
| openai_compatibility::http_models_endpoint_advertises_only_formal_ai | stale test | #1591 (ab89e8c5d) added `formal-ai-embeddings` | high |
| openai_compatibility::chat_completion_refuses_tool_call_without_agent_mode | product regression | #1590 forced-function passthrough also returned classified shell tools as client function calls | medium-high |
| associative_packages::chat_tool_gate_requires_agent_mode_and_package_permission | stale test | #1590: a forced `tool_choice` naming a plain client function is returned to the caller, so the agent-mode gate is now exercised by an unforced tool | medium-high |
| summarization_pipeline::summarization_request_summarizes_inline_conversation_content | product regression | `handle_summarization_request` (#1174) claimed "Summarize conversation: ..." before `conversation_memory` | medium-high |
| code_generation::single_turn::rust_hello_world_composes_a_stdout_program | non-hermetic | 10a6f2855 documented-program rediscovery: the answer is "rediscovered" when the Rust book is cached or reachable, otherwise "not probed" | medium |
| code_generation::single_turn::russian_list_files_in_rust_returns_program, english_list_files_in_python_returns_program | stale test | 9adbb43f5 status wording ("not probed in this environment") | high |
| code_generation::single_turn::russian_program_request_with_unknown_task_is_not_unknown | stale test | 9b1ab09ca: factorial is operand-bound, so a request with no operand asks for it (intent `write_program`, guidance `request-factorial-operand`) | high |
| code_generation::task_catalog::factorial_operand_excludes_the_number_of_requested_functions | non-hermetic | Wikifunctions Z13863 recurrence synthesis composes when captures are available (shared source cache or a live read), otherwise it refuses. In CI, hi composed and en refused | medium |
| code_generation_blueprint::budget_calculator_blueprint_covers_supported_prompt_languages | product regression | 0f4a76eac removed the city-name keywords; Russian "стоимости жизни и аренде" no longer reached `city_costs` | medium-high |
| prompt_variations::hello_world_matrix_emits_a_code_block_per_language | non-hermetic | same as rust_hello_world (10a6f2855) | medium |
| prompt_variations::brainstorming_intent_routes_to_brainstorm_handler | product regression | 847753b12 reads the brainstorm topic from the method-dispatch `normalized` argument. That argument is `prompt.to_lowercase()`, so "local-first ai helper." leaked into the items, trailing period included | medium-high |
| prompt_variations::factual_qna_matrix_records_per_language_evidence | stale test | 17aabb4d0 / bf813658e changed the test to call the fact handler directly (`fact_answer`). Only `UniversalSolver::solve` records the `language:<slug>` link (solver.rs `log.append("language", ...)`) | medium-high |
| conversational_reverification::independent_questions_are_answered_in_source_order_in_every_language | product regression | `meta_executor::split_request` matched the Hindi connective "या" (or) inside "क्या" ("what"). The character before it is a virama (U+094D), which is a combining mark and not `Alphabetic`, so the word-boundary check passed. The inner solve of "आप क्या कर सकते हैं?" split into two more sub-impulses (5 instead of 3). Present since #1550 (27749f7e0 / d21b8bf62) | medium-high |
| reasoning_paths_procedures::* (7 tests) | non-hermetic | 7a80e4f1a (#1632) always probes wikiHow. With read access on by default (c0e480a36), live guides replace the refusals and the fallback-plan evidence the tests assert | medium-high |

## Fixes

- **Recipe paths.**
  - `data/meta/agentic-coding-recipe.lino`: `fn_classify_tool` now points to `src/agentic_coding/capability_router/tool_classification.rs`, and `surface_cli_subcommand` to `src/cli_solver.rs`.
  - `data/meta/procedural-howto-recipe.lino`: `fn_answer_markdown` now points to `src/how_to_guide/render.rs`.
  - `data/meta/concept-lookup-recipe.lino`: step-1 now points to `concept_lookup/unknown_surfaces.rs`.
  - The tests' expected paths were updated to match, in `concept_lookup_meta_algorithm.rs` and `budget_search_meta_algorithm.rs`.
- **`specification/mod.rs`**: added `cnl-settings.jsx` to the JSX module list.
- **`method_registry.rs`**: expects 18 contextual overrides instead of 14.
- **`openai_compatibility.rs`**: the full model list is now `formal-ai` plus `formal-ai-embeddings`. A new assertion checks that `formal-ai` is the only chat model, so the test keeps its original intent.
- **Forced client functions (#1590)**:
  - Change: `rust/src/protocol/client_functions.rs` no longer treats a name as a client function when `planner::tool_capability(name)` classifies it, so classified tools such as shell still go through the agent-mode gate.
  - Same change in the twins `js/server/client-functions.mjs` and `ts/server/client-functions.mts`.
  - Parity row "classified shell gate" added to `data/parity/issue-1590-forced-functions.json`.
  - `associative_packages.rs` advertises the calculator tool without forcing `tool_choice`, so it still tests the gate.
- **Summarization**:
  - `summarization_request.rs` returns `None` when the command head mentions `ROLE_CONVERSATION_REFERENCE` or `ROLE_CONVERSATION_SUMMARY_PHRASE`.
  - Same change in `js/worker/formal_ai_worker_text_transform.js`, with the byte-identical TS twin updated.
- **Code-generation tests**:
  - `single_turn.rs`: rust_hello_world accepts either the "not probed" or the "rediscovered" status. The list-files tests expect "not probed". The Russian unknown-task test now asserts intent `write_program`, the language and task program parameters, the Russian factorial operand question, and no code block.
  - `task_catalog.rs`: either a verified composition (`def factorial(n)` recurrence plus wikifunctions.org) or the exact refusal is accepted. The test always asserts `range(1, 2)` is absent, which is its core intent: the artifact count is not bound as the operand.
- **Budget blueprint**: added the keywords `аренд` and `стоимости жизни` to `city_costs` in four files:
  - `rust/src/coding/blueprint_data.rs`
  - `rust/tests/source/coding/blueprint_data.rs` (the mirror)
  - `js/worker/formal_ai_worker_program_plans.js`
  - `ts/worker/formal_ai_worker_program_plans.ts`

  A python simulation detects all 5 capabilities in en, ru, hi and zh.
- **hello_world_matrix** (`prompt_variations.rs`): for Rust, when the answer cites the Rust book URL, the expected text switches to the localized en/ru "rediscovered" status and output-heading lines.
- **brainstorming**: `rust/src/solver_handlers/benchmark_prompts.rs::try_brainstorming_request` now reads the topic from `crate::engine::normalize_prompt(normalized)`. The answer becomes "local first ai helper", matching the test and the browser twin, which reads normalized text. Matching for triggers, category and count is unchanged.
- **factual_qna per-language evidence** (`prompt_variations.rs`): the language tag is asserted on an offline `UniversalSolver` turn (`SolverConfig { offline: true, .. }`). That is where the solver records it, and the test stays hermetic. The publisher-replay assertions remain in `factual_qna_matrix_records_wikidata_anchor`.
- **Hindi decomposition**: the connective word-boundary check now treats combining marks (Unicode Mn/Mc/Me) as word characters.
  - Rust: `rust/src/meta_executor.rs`, new `word_character` using `unicode_general_category`, already a dependency.
  - JS: `js/agentic/crate/meta_executor.mjs`, new `isWordCharacter` using `\p{M}`.
  - TS twin `ts/agentic/crate/meta_executor.mts`, plus the generated worker module `js/worker/formal_ai_worker_crate_meta_executor.js` (regenerated) and its TS copy.
  - Checked with `splitRequest`: "आप क्या कर सकते हैं?" → no split (before the fix: `["आप क्", "कर सकते हैं?"]`). "चाय या कॉफी?", "tea or coffee?" and "Кто ты или что ты?" still split as alternatives.
  - No meta-executor parity or census fixture contains the affected text.
- **reasoning_paths_procedures** (fork): the tests use `offline_solver()` / `offline_answer()` (`SolverConfig { offline: true, .. }`).

## Shared or widely used files touched

- `rust/src/protocol/client_functions.rs`, `js/server/client-functions.mjs`, `ts/server/client-functions.mts`, `data/parity/issue-1590-forced-functions.json`
- `rust/src/solver_handlers/summarization_request.rs`, `js/worker/formal_ai_worker_text_transform.js`, `ts/worker/formal_ai_worker_text_transform.ts`
- `rust/src/coding/blueprint_data.rs` (and the `rust/tests/source` mirror), `js/worker/formal_ai_worker_program_plans.js`, `ts/worker/formal_ai_worker_program_plans.ts`
- `rust/src/meta_executor.rs`, `js/agentic/crate/meta_executor.mjs`, `ts/agentic/crate/meta_executor.mts`, `js/worker/formal_ai_worker_crate_meta_executor.js`, `ts/worker/formal_ai_worker_crate_meta_executor.ts`
- `rust/src/solver_handlers/benchmark_prompts.rs`

## Follow-ups

- The hosted worker receipt or bundle may need regenerating, because worker sources changed (`program_plans`, `text_transform`, `crate_meta_executor`). That regeneration only runs in hosted CI and was not run here.
- `node scripts/check-typescript-runtime.mjs` currently fails on `ts/agentic/evidence_record.mts`. That file belongs to another agent's change (`js/agentic/evidence_record.mjs`), not to this cluster.
- Several specification tests rely on `FormalAiEngine.answer`. Read access is on by default, so these tests go live or read the shared cache. Moving the specification suite to an offline solver, or to an isolated cache, would remove the class of non-hermetic failures fixed case by case here.
