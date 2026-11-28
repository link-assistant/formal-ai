# Rust test failures on main (run 45636781871, commit 513e4d9)

How these were reproduced:

- Locally, using the prebuilt test executables from run 45636781871. Their
  `CARGO_MANIFEST_DIR` was patched to a symlink pointing at this checkout.
- Each run used `ulimit -v 2000000`, `--test-threads=1`, `--exact`, and an
  exclusive `FORMAL_AI_SOURCE_CACHE_DIR`/`FORMAL_AI_CACHE_DIR` (helper
  `rt.sh <target> --exact <test>`).
- Each test ran in the default environment and with `FORMAL_AI_NETWORK=denied`.
- The binaries contain the 513e4d9 code, so they cannot exercise the edits
  below. No local cargo build was possible (3 GB cgroup).
- Every changed Rust file passes `rustfmt --edition 2024 --check`.

Two mechanisms explain most of the failures:

- **Read network is on by default.** Since c0e480a36 (#1582), a default solver
  in CI asks real publishers (Wikipedia, Wikifunctions, MDN, wikiHow, ...).
- **The source cache is shared across a CI job.** `source_cache_root()` falls
  back to `FORMAL_AI_SOURCE_CACHE_DIR`, then `FORMAL_AI_CACHE_DIR`, then
  `data`, and one cache serves the whole job. A capture written by one test
  changes the answer another test gets, regardless of execution order.

## 1. `issue_1588_intent_spans::document_intent_and_bound_quantities_match_worker_fixtures`

- **Symptom:** the Rust intent spans differ from the worker fixture for "how
  many days are in 2 years?".
  - The prompt is routed to `compose_from_sources` / `concept_measurement_lookup`.
  - It should be a unit conversion, which the JS worker does.
- **Reproduction:** `rt.sh unit --exact issue_1588_intent_spans::document_intent_and_bound_quantities_match_worker_fixtures`
  fails in both environments, so it is deterministic and not network-dependent.
- **Introducing commit:**
  - 91a64e8d3 added the fixture row.
  - The mismatch was hidden by an earlier failure in the same test until #1637
    fixed that failure.
- **Root cause:** the meta-method gate for the source-composition arms only
  checks the lexicon unit list. "years" is an SI-derived time unit that is not
  in that list, so the measurement lookup claims the prompt first.
- **Fix (product):** `rust/src/meta_method_dispatch.rs`.
  - The `compose_from_sources | concept_measurement_lookup` arm now probes
    `solver_handlers::handle_unit_conversion`.
  - It declines when that probe records a `unit_conversion:derivation`.
  - The fix is placed here, not in `unit_conversion.rs`, to keep the
    core-boundary ledger baseline.
- **Verification:** fixed, unverified. A rebuild is needed.
- **Narrowest PR-lane check:** the `issue_1588_intent_spans` unit filter. It is
  deterministic and takes seconds.

## 2. `coding::catalog::lino_parity::{lino_seed_mirrors_every_catalog_template, lino_seed_has_no_extra_templates}`

- **Symptom:** after PR #1614 these templates have no 1:1 LiNo seed entry:
  - catalog templates rendered by a renderer;
  - templates composed from shared builders.
- **Reproduction:** `rt.sh source --exact coding::catalog::lino_parity::lino_seed_mirrors_every_catalog_template`
  fails in both environments, deterministically.
- **Introducing commit:** 10a6f2855 (PR #1614), which moved templates to
  renderer-built code.
- **Root cause:** the parity test compared raw template strings. Since #1614,
  the seed mirrors the rendered program.
- **Fix (test, intended change):**
  `rust/tests/source/source_tests/coding/catalog/mod/lino_parity.rs`.
  - Parity now uses `rendered_template_code` instead of raw template strings.
  - The composed-template count is asserted explicitly.
- **Verification:** fixed, unverified. A rebuild is needed.
- **Narrowest PR-lane check:** the source binary's `coding::catalog::lino_parity`
  filter. Run it in any PR that touches `rust/src/coding/catalog*` or
  `data/seed/*catalog*`.

## 3. `issue_1158_pull_request_steps::the_hive_mind_prompt_reads_feedback_then_titles_describes_and_readies_the_pull_request`

- **Symptom (CI shard 7/8):** the test started around 15:58:24 and failed at
  15:59:19, about 55 s later. The session answer ended with the recipe-resume
  scope refusal (`data/meta/recipe-resume.lino`) instead of "the pull request
  is ready."
- **Reproduction:** it does not reproduce locally.
  - Default environment: `ok` in 246.50 s.
  - `FORMAL_AI_NETWORK=denied`: `ok` in 227.85 s.
  - Siblings `issue_1157` and `issue_1160` passed in the same CI shard.
- **Introducing commit:** none identified for the logic. The likely trigger is
  c0e480a36 (#1582, read network on by default) combined with the shared
  per-job cache.
- **Root cause (probable, not proven):**
  - The planner re-derives the recipe on every turn
    (`agentic_coding/general_execution.rs`, `solve_for_execution`).
  - CI runs tests in parallel against one shared, network-backed source cache.
  - If a concurrent capture or a live answer changes the derived recipe between
    turns, recipe resume sees an ownership or observation mismatch.
  - `recipe_resume.rs` then returns `gap("scope-refusal")`.
- **Fix (test hermeticity):**
  `rust/tests/unit/issue_1158_pull_request_steps.rs` now replays the test
  through `offline_replay::in_network_denied_child`, with read access denied
  and an exclusive, empty cache.
- **Verification:**
  - Fixed, unverified by build.
  - The network-denied behaviour it replays was verified to pass: 227.85 s
    with the 513e4d9 binary.
- **Narrowest PR-lane check:** none is reliable, because the failure is
  order- and network-dependent. Per-test cache isolation (see the workflow
  proposals) removes this class of failure.

## 4. `minimal_core_and_seed_metadata::coding_path_has_complete_metadata_and_every_other_gap_is_data`

- **Symptom:** `left: 1081, right: 1073` (the seed metadata count pin).
- **Reproduction:** `rt.sh unit --exact minimal_core_and_seed_metadata::coding_path_has_complete_metadata_and_every_other_gap_is_data`
  fails deterministically with `1081 vs 1073`.
- **Introducing commits:** PR #1614, PR #1599 and PR #1581 added seed rows
  intentionally, without moving the pin.
- **Fix (expectation update, intended):** the pin is now 1081. The comment
  cites those PRs.
- **Verification:** verified. The binary reports exactly 1081, the new
  expected value.
- **Narrowest PR-lane check:** this unit filter. Run it in any PR that touches
  `data/seed/**`.

## 5. `docs_requirements::issues::full_range_coding_artifacts::every_test_the_traceability_rows_cite_exists`

- **Symptom:** a traceability row cites a test function that was renamed.
- **Reproduction:** the `rt.sh unit --exact ...` run fails deterministically.
- **Introducing commit:** the rename of the cited test, done without updating
  its requirement rows.
- **Fix (docs):** the citation now names
  `a_framework_named_coding_request_keeps_the_framework` in:
  - `docs/requirements-traceability-continuation.md`;
  - `docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md`;
  - `docs/requirements/assembled/coding-and-agents.md`.
- **Verification:** verified with the binary plus a simulation of the citation
  check.
- **Narrowest PR-lane check:** the `docs_requirements` unit filter. Run it in
  any PR that renames a `#[test]` fn.

## 6. `solver::issue_938_meta_algorithm_builder::all_coding_surfaces_execute_one_trace_shape`

- **Symptom:** "missing shared builder". In the default environment, a coding
  surface answered with a rediscovered Rust-book program that has no
  `algorithm_construction` trace step.
- **Reproduction:**
  - Default environment: `FAILED` in 45.36 s.
  - `FORMAL_AI_NETWORK=denied`: `ok` in 18.61 s.
- **Introducing commit:** 10a6f2855 (PR #1614, operand-program path), exposed
  by c0e480a36 (#1582, read network on by default).
- **Root cause:** the operand-program catalog branches never recorded
  `record_algorithm_construction`. Only the offline route happened to reach a
  path that records it.
- **Fix (product):** `rust/src/coding/operand_program.rs`.
  - New `record_catalog_construction` records the shared builder step for a
    catalog `ProgramReply::Program` that has no rule-synthesis candidate.
  - It is called in both the literal-operand branch and the documented branch.
  - The test is deliberately left non-hermetic, because the trace shape should
    hold online too.
- **Verification:** fixed, unverified. A rebuild is needed.
- **Narrowest PR-lane check:** this solver filter, in the default environment.

## 7. `specification::code_generation::task_catalog::factorial_operand_excludes_the_number_of_requested_functions`

- **Symptom (CI spec shard 2/4, log about line 3563):**
  - For the prompt "Python में एक फ़ंक्शन लिखो जो 5 का फ़ैक्टोरियल लौटाए", the
    answer is the composed `def factorial(n)` recurrence from Wikifunctions
    Z13863, not the catalog binding.
  - The trace shows live fetches to hi.wikipedia and wikifunctions.org.
- **Reproduction:**
  - Default environment: `FAILED` in 19.34 s, with the same live fetches and
    the same answer.
  - `FORMAL_AI_NETWORK=denied` with an empty cache: `ok` in 4.43 s.
  - CLI cross-check (online): "Write one function in Python for factorial of
    five" gets the recurrence. "Write 2 functions in Python to calculate
    factorial of 5" gets the catalog `range(1, 6)` program.
- **Introducing commits:**
  - c30b54f15 (gate from 4cae2fecb) probes the publisher recurrence before
    catalog binding, for any Python function request.
  - 9b1ab09ca added the bound loop after c0e480a36 had turned the network on.
  - 15ae736ab (#1637) relaxed the first loop into a tolerant composed-answer
    branch, which unmasked the bound loop.
- **Root cause:** the test is a catalog-binding contract, but online, or with a
  shared cache holding the capture, the recurrence wins. Both the Rust and JS
  twins prefer the recurrence by design (`js/worker/formal_ai_worker_solve.js`).
- **Fix (test hermeticity):**
  `rust/tests/unit/specification/code_generation/task_catalog.rs`.
  - Added an `in_network_denied_child` replay.
  - The first loop asserts the exact refusal again. The composed recurrence is
    already covered by `solver::issue_1173_route3_recurrence` and
    `issue-1396-native.test.mjs`.
- **Verification:** fixed, unverified by build. The network-denied, empty-cache
  behaviour was verified to pass with the binary (4.43 s).
- **Open product question:** should a bound literal operand ("5") outrank the
  publisher recurrence? This needs an owner decision. It is not changed here.
- **Narrowest PR-lane check:** this spec filter.

## 8. `specification::prompt_variations::hello_world_matrix_emits_a_code_block_per_language`

- **Symptom (CI spec shard 3/4, log about line 3558):** the JavaScript row
  answered from a cached MDN `console/index.md` capture with no code block.
- **Reproduction:**
  - With a fresh cache, the test passes.
  - Seeding a shared cache with the CLI prompt "Show me hello world in
    JavaScript", which captures MDN raw `console/index.md`, makes it `FAILED`
    in 1.47 s with the CI message. That confirms cross-test cache
    contamination.
- **Introducing commit:** c0e480a36 (#1582), with the shared per-job cache.
  The matrix itself is older.
- **Fix (test hermeticity):**
  `rust/tests/unit/specification/prompt_variations.rs` replays through
  `offline_replay::in_child`, offline with an exclusive cache.
- **Verification:** fixed, unverified by build.
- **Narrowest PR-lane check:** this spec filter, run after a seeding prompt.
  A per-test cache removes the class.

## Is a job-wide `FORMAL_AI_NETWORK=denied` safe? Not as-is.

Evidence:

- **The denial also blocks loopback.** `computer_use/http.rs:26` (`target()`)
  returns `computer_use_network_denied` before any origin check. These tests
  start a loopback `fixtures::Publisher` and send HTTP steps through it, so
  they would fail under a job-wide denial:
  - `rust/tests/issue_707_mcp.rs` `mcp_executes_and_verifies_all_ten_plans_in_agent_mode`.
    Its child inherits the env and sets only `FORMAL_AI_COMPUTER_USE_HTTP_ORIGIN`.
  - `rust/tests/issue_707_computer_use.rs`, lines 84 and 452.
  - `rust/tests/issue_707_generalization.rs`, line 147.
- **Tests that are safe:**
  - `integration/with_formal_ai/issue_1582.rs` already calls
    `env_remove("FORMAL_AI_NETWORK")` and owns its default-on case.
  - The other `TcpListener` tests serve formal-ai or fake agent protocols. They
    do not go through the read-network gate.
  - The `live_*` tests (`issue_1163_1164_captured_pages`,
    `issue_1164_pascal_from_documentation`, `issue_991`) are opt-in through
    `FORMAL_AI_LIVE_FETCH=1`, which no workflow sets.
- **Recommendation:** either exempt the explicitly configured loopback
  computer-use origin from the read-network deny, or `env_remove` it in those
  four tests. Then run the test job with `FORMAL_AI_NETWORK=denied` and an
  exclusive cache per job. A per-test cache would be better still. A full
  denied PR-lane run is still needed to list any test that silently relies on
  live answers.

## Slow tests and the 1080 s budget

Three shards were killed at the 1080 s budget:

| Shard | Duration | Tests still running at the kill |
| --- | --- | --- |
| full (1) | 1082 s | `coding_research_production::canonical_native_production_route_handles_cold_held_out_typescript_and_reuses_without_writes`<br>`meta_executor_census::every_specification_prompt_runs_the_recursive_procedure`<br>`semantic_intent_routing::web_search_routes_action_variations_in_every_supported_language` |
| 3/8 | 1083 s | `issue_1562_cited_programs::the_three_reports_write_run_and_commit_nothing_and_name_the_defect` |
| 6/8 | 1082 s | `issue_1138_family_migration::held_out_family_paraphrases_route_to_the_family_interpreter`<br>`issue_701_learning_adoption::the_adoption_ledger_records_a_real_capability_delta_in_every_language`<br>`seed::issue_1138_intent_phrase_migration::held_out_web_search_paraphrases_reach_the_canonical_search_answer`<br>`seed::issue_699_handler_migration::held_out_number_constraint_paraphrases_are_data_driven` |

Other shards finished in 593-801 s. Over-60 s tests in the other shards are
mostly:

- `issue_558_source_links::serialized_owned_source_shard_*` (5-11 min each);
- the `issue_1133` / `issue_1157` / `issue_1160` Hive Mind sessions;
- `coding_discovery::*` composition;
- `issue_701_learning_adoption`.

Local timings with the 513e4d9 binary, single-threaded:

| Test | Default environment | `FORMAL_AI_NETWORK=denied` |
| --- | --- | --- |
| `seed::issue_1138_intent_phrase_migration::held_out_web_search_paraphrases_reach_the_canonical_search_answer` | more than 400 s (killed by timeout) | `ok` in 7.35 s |
| `seed::issue_699_handler_migration::held_out_number_constraint_paraphrases_are_data_driven` | 16.90 s | 6.19 s |
| `issue_1138_family_migration::held_out_family_paraphrases_route_to_the_family_interpreter` | more than 240 s | more than 240 s |

- The web-search test is network-bound: the web-search handler does live
  searches whenever `read_network_allowed()` is true (`live_search.rs:19`).
- `issue_1138_family_migration` is CPU-bound, not network-bound.
- `semantic_intent_routing::web_search_routes_action_variations_in_every_supported_language`
  and `meta_executor_census` (every specification prompt) use the same default
  solver with no offline guard, so they make live calls too.

So the 6/8 and full-1 timeouts are at least partly live web-search latency.
That latency was introduced by default-on read access (#1582). None of the
tests that were killed while still running set an offline or denied guard.
