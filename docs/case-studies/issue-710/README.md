# Issue #710 — Dropped-Requirements Regression Backlog

Issue [#710](https://github.com/link-assistant/formal-ai/issues/710) is the
tracked regression backlog produced by the 2026-07-14 full-history requirement
audit: every closed issue (all 329) and every merged pull request (all 317) was
re-read against the maintainer's original requirements, konard's follow-up
comments, and the delivery evidence in each thread, then cross-checked against
the repository state at `main` v0.285.0.

## Raw data

The audit reports live in [`raw-data/`](raw-data/):

| File | Scope |
| --- | --- |
| `report-open-issues.md` | Digest of all 31 open issues (requirements, themes, blocking graph) |
| `report-open-prs.md` | State of all 17 open PRs and their unaddressed feedback |
| `report-closed-issues-1-350.md` | Per-issue verdicts for the 183 closed issues ≤ #350 |
| `report-closed-issues-351-plus.md` | Per-issue verdicts for the 146 closed issues > #350, with the consolidated dropped-requirements list |
| `report-merged-prs-first-half.md` | konard-comment audit of merged PRs #2–#328 |
| `report-merged-prs-second-half.md` | konard-comment audit of merged PRs #328–#683, verified against `main` |
| `report-problem-solving-repo.md` | Digest of the konard/problem-solving methodology this project should follow |
| `local-doc-consistency-findings.md` | Documentation inconsistencies found and fixed in the same pass |

## Headline findings

- Of 183 closed issues ≤ #350, only ~15% show clear in-thread delivery
  evidence; of 146 closed issues > #350, roughly half were partially addressed.
- The dominant failure mode is **silent scope-narrowing**: the reported prompt
  gets fixed while the attached generalization, benchmark, or integration
  requirement is dropped.
- Recurring dropped themes: generalization vs. memoization, "all languages"
  narrowed to four, real external benchmarks replaced by local proxies,
  loopback tests instead of real agentic clients, deferred work despite
  "defer nothing" instructions, and standing process clauses (case studies,
  upstream filings) skipped.

## Follow-up structure

The audit produced the E56–E68 planning batch
([#698](https://github.com/link-assistant/formal-ai/issues/698)–[#710](https://github.com/link-assistant/formal-ai/issues/710)),
all sub-issues of [#651](https://github.com/link-assistant/formal-ai/issues/651)
with explicit blocked-by relationships. Issue #710's checklist enumerates the
smaller silently-dropped items; the twelve sibling issues own the large
capability gaps. `ROADMAP.md` gained a requirement-level status table
(done / partial / not done) in the same pass.

## 2026-08-10 follow-up re-verification

The checklist below was re-run against current `main` v0.337.0 after every
focused follow-up from the 2026-08-01 pass had merged. No verdict is inferred
from an issue or pull request being closed: each changed row points at a current
production-path regression, and the folder-routing complaint from the issue
conversation was repeated through a live TCP server. The allowed verdict
vocabulary is deliberately closed:
`works-now`, `still-broken`, `superseded`, and `blocked-upstream`. A
`works-now` row names a production-path regression; a `still-broken` row names
an open focused owner. No row in this audit currently has enough evidence for a
`blocked-upstream` verdict.

| # | Area | Audited requirement | Verdict | Current evidence or focused owner |
| ---: | --- | --- | --- | --- |
| 1 | Chat | Conversation-history recall | `works-now` | [`conversation_history.rs`](../../../tests/unit/specification/conversation_history.rs), `solve_with_history_searches_dialog_history_in_russian` and the four-language `previous_user_question_recall_skips_meta_turns_in_supported_languages`. |
| 2 | Chat | Russian identity and capabilities | `works-now` | [`multilingual.rs`](../../../tests/unit/specification/multilingual.rs), `russian_identity_question_returns_identity_intent`, `russian_capabilities_answer_is_in_russian`, and `russian_more_capabilities_follow_up_uses_history_without_repeating_web_search`. |
| 3 | Chat | Multi-statement and many-question composition | `works-now` | [`issue_710.rs`](../../../tests/unit/specification/issue_710.rs), `independent_questions_are_answered_in_source_order_in_every_language`; browser parity is pinned in [`issue-710.spec.js`](../../../tests/e2e/tests/issue-710.spec.js). |
| 4 | Chat | Context-qualified questions such as IIR in ML | `works-now` | [`multilingual.rs`](../../../tests/unit/specification/multilingual.rs), `russian_iir_in_ml_returns_context_aware_concept_lookup` plus English, Hindi, and Chinese counterparts. |
| 5 | Chat | Typo tolerance, clarification, and full-path fuzzy matching | `works-now` | [`issue-343.spec.js`](../../../tests/e2e/tests/issue-343.spec.js) and [`calculator_delegation.rs`](../../../tests/unit/specification/calculator_delegation.rs), `calculator_explains_fuzzy_calculate_typo` and `calculator_fuzzy_prefix_is_not_limited_to_one_spelling`. |
| 6 | Chat | Antiregime and false-totality definition class | `works-now` | [`multilingual.rs`](../../../tests/unit/specification/multilingual.rs), `russian_antiregime_question_returns_seeded_concept_lookup` and `false_totality_questions_resolve_across_supported_languages`. |
| 7 | Chat | Folder-listing prompt variants | `superseded` | General capability routing in [#745](https://github.com/link-assistant/formal-ai/issues/745), [#758](https://github.com/link-assistant/formal-ai/issues/758), and merged [PR #850](https://github.com/link-assistant/formal-ai/pull/850); pinned by [`issue_745.rs`](../../../tests/unit/issue_745.rs), `directory_listing_routes_shell_variations_in_every_supported_language`, and the real-server `live_http_server_routes_reported_folder_variants_to_one_shell_contract` in [`issue_758_capability_routing.rs`](../../../tests/integration/issue_758_capability_routing.rs). The broader grounded task ladder also re-runs 24/24 through [`run_ladder.sh`](../../../experiments/issue_840_task_ladder/run_ladder.sh). |
| 8 | Chat | Target-less modification asks one question | `works-now` | [`issue_710.rs`](../../../tests/unit/specification/issue_710.rs), `ambiguous_modifications_ask_exactly_one_question_in_every_language`, with the same worker assertion in [`issue-710.spec.js`](../../../tests/e2e/tests/issue-710.spec.js). |
| 9 | Chat | Multiple deterministic free-time replies | `works-now` | [`issue_710.rs`](../../../tests/unit/specification/issue_710.rs), `free_time_answers_are_prompt_stable_but_not_one_canned_reply`; variants are seed records rather than runtime randomness. |
| 10 | Chat | Assistant name set/read and attribution | `works-now` | [`issue_710.rs`](../../../tests/unit/specification/issue_710.rs), `assistant_name_can_be_set_and_recalled_in_every_language`; attribution remains pinned by [`issue-157.spec.js`](../../../tests/e2e/tests/issue-157.spec.js). |
| 11 | Localization | Issue #292 rules, answer-language, parity, and Markdown asks | `works-now` | [`behavior_rules.rs`](../../../tests/unit/specification/behavior_rules.rs), `behavior_rules_list_answer_is_localized_for_supported_languages`; [`multilingual.spec.js`](../../../tests/e2e/tests/multilingual.spec.js), `reported Russian behavior-rule list is localized and markdown-safe`; generated parity scripts live under [`tests/e2e/scripts`](../../../tests/e2e/scripts/). |
| 12 | Localization | Thinking localization outside the browser UI | `works-now` | [`issue_889_thinking_surfaces.rs`](../../../tests/issue_889_thinking_surfaces.rs) drives every registered language through CLI thinking, OpenAI Chat Completions reasoning, Anthropic thinking blocks, and stored step summaries; [`issue_889_thinking_seed.rs`](../../../tests/unit/issue_889_thinking_seed.rs) pins the shared seed vocabulary. |
| 13 | Localization | Collapsed thinking animation and top placement | `works-now` | [`issue-488.spec.js`](../../../tests/e2e/tests/issue-488.spec.js), `shows collapsed human-readable thinking by default and expands details` and `localizes thinking preview and detail settings across supported languages`; narrative order is pinned in [`issue_676_thinking_narrative.rs`](../../../tests/unit/issue_676_thinking_narrative.rs). |
| 14 | Knowledge | Translate formal proofs to programming languages | `works-now` | [`issue_890.rs`](../../../tests/unit/issue_890.rs), `whole_issue_890_workflow_solves_translates_and_executes`, runs one solved proof through the general Rust and Python translators and executes both results; the registered-language request matrix is pinned alongside it. |
| 15 | Knowledge | At least 50 verified equation types | `works-now` | [`equation_corpus.rs`](../../../tests/unit/specification/equation_corpus.rs), `issue_891_equation_corpus_solves_every_type`, replays 72 distinct machine-readable types through `FormalAiEngine` with a minimum-50 and non-decreasing 72-pass ratchet; recorded limitations must continue to decline rather than fabricate. |
| 16 | Knowledge | Compose calculations with other instructions | `works-now` | [`calculator_delegation.rs`](../../../tests/unit/specification/calculator_delegation.rs), `embedded_request_variations`, compound-interest continuation tests, and the issue-710 independent-question specification. |
| 17 | Knowledge | Word problems beyond train meeting | `works-now` | [`calculator_delegation.rs`](../../../tests/unit/specification/calculator_delegation.rs), `fibonacci_word_problem_reduces_to_calculator_expression` and `box_relation_word_problem_resolves_total_with_reasoning`. |
| 18 | Knowledge | Current films in release order, not a stale seed | `works-now` | [`issue_892.rs`](../../../tests/unit/specification/issue_892.rs) transcribes a timestamped checked-in Wikidata capture and exercises current, stale, future, and undated releases through the production answer path in every registered language. |
| 19 | Knowledge | Closest contextual pronoun resolution | `works-now` | [`issue_465.rs`](../../../tests/unit/specification/issue_465.rs), `pronoun_followup_resolves_prior_rust_topic_for_creator_question`, with multilingual fact availability. |
| 20 | Knowledge | How-to multi-source synthesis and seven-day availability cache | `still-broken` | [#709](https://github.com/link-assistant/formal-ai/issues/709) delivered statement-level source fusion, but the issue-444 case study still identifies reasoned procedural synthesis, recursive capture, per-service accessibility state, and real-service QA replay as unfinished. Focused owner [#991](https://github.com/link-assistant/formal-ai/issues/991) now carries that exact residual contract. |
| 21 | Knowledge | Iterative two-file summary validation and 80% quality bar | `works-now` | [`issue_893_summarization_validation.rs`](../../../tests/unit/specification/issue_893_summarization_validation.rs) samples two real repository files per seeded iteration until stable/bounded, runs embedded grammars through the production summarizer, and enforces the published 80% ratchet. |
| 22 | Knowledge | Interior/plain-capitalized entity reasoning class | `works-now` | [`issue_571.rs`](../../../tests/unit/issue_571.rs), `external_entity_questions_route_to_web_search_by_reasoning_not_vocabulary`; lower-case Tesla routing remains pinned in [`multilingual.spec.js`](../../../tests/e2e/tests/multilingual.spec.js). |
| 23 | Platform | Calendar interchange and Apple/Google/Microsoft flows | `works-now` | [`calendar_ics.rs`](../../../src/solver_handlers/calendar_ics.rs) emits RFC 5545 accepted by Apple Calendar, Outlook, and Google Calendar plus a Google insertion URL; [`issue-404.spec.js`](../../../tests/e2e/tests/issue-404.spec.js) exercises the production worker. |
| 24 | Platform | Optional gated OCR and image transcription | `works-now` | [`issue-493.spec.js`](../../../tests/e2e/tests/issue-493.spec.js), `uses OCR text to flag the false ETH 2024 price claim`, plus multilingual/generalization coverage and the explicit OCR preference gate. |
| 25 | Platform | E2E against deployed GitHub Pages | `works-now` | [`workflow_release.rs`](../../../tests/unit/ci-cd/workflow_release.rs), `pages_e2e_uses_deployment_output_url` and `pages_deploy_is_pinned_and_live_e2e_waits_for_matching_deployment`. |
| 26 | Platform | Four-template CI comparison and upstream filings | `works-now` | The revalidated ledger in [`REPORT.md`](../issue-479/template-comparison/REPORT.md) gives every confirmed template gap an owning upstream URL; [`docs_requirements_issue_894.rs`](../../../tests/unit/docs_requirements_issue_894.rs), `issue_894_every_confirmed_finding_carries_an_upstream_filing_url`, rejects any confirmed or ready-to-file row without one. |
| 27 | Platform | Published coverage with a non-decreasing ratchet | `works-now` | [`coverage.yml`](../../../.github/workflows/coverage.yml) publishes separate Rust and browser reports and gates both against [`baseline.json`](../../../coverage/baseline.json); [`workflow_coverage.rs`](../../../tests/unit/ci-cd/workflow_coverage.rs) pins the non-decreasing, separately measured ratchet and upload contract. |
| 28 | Platform | Gemini headless tools | `works-now` | Superseding all-client work [#671](https://github.com/link-assistant/formal-ai/issues/671) / [PR #814](https://github.com/link-assistant/formal-ai/pull/814) records Gemini headless `read_file` requests and tool calls in [`recorded/gemini/read-file.jsonl`](../../../experiments/agentic_cli_matrix/recorded/gemini/read-file.jsonl). |
| 29 | Platform | macOS signed/notarized auto-update production path | `works-now` | [`issue-548.spec.js`](../../../tests/e2e/tests/issue-548.spec.js) pins version/event/localization behavior; [`desktop-release.yml`](../../../.github/workflows/desktop-release.yml) owns signing, notarization, update metadata, and explicit ad-hoc fallback diagnostics. |
| 30 | Platform | link-foundation/start and command-stream adoption | `still-broken` | `start-command` is now installed and used by the Docker-in-Docker production contract, pinned by [`docker_runtime.rs`](../../../tests/unit/docker_runtime.rs). Desktop, VS Code, and Rust orchestration still use custom process runners with no production `command-stream` dependency; focused owner [#990](https://github.com/link-assistant/formal-ai/issues/990) tracks only that remaining half. |
| 31 | Platform | web-search/web-capture as real components | `works-now` | [`issue_896_component_boundaries.rs`](../../../tests/unit/issue_896_component_boundaries.rs) executes both published components through the native production boundary, pins failure/fallback behavior and build budgets, and [`issue-896.spec.js`](../../../tests/e2e/tests/issue-896.spec.js) exercises web-capture in the browser HTTP path. |
| 32 | Platform | Iframe pre-check and external-link actions | `works-now` | [`multilingual.spec.js`](../../../tests/e2e/tests/multilingual.spec.js), `GitHub navigation suggests an external link without iframe preview` and `Navigation previews URLs when frame policy allows embedding`. |

Totals: **29 `works-now`**, **2 `still-broken`**, **1 `superseded`**, and
**0 `blocked-upstream`**. The eight completed follow-ups are credited only after
their current regressions passed. The two aggregate requirements that remain
partly unimplemented are linked to newly focused owners rather than to their
already-closed prerequisite issues.

## Recovered conversational regressions

Four minimum reproductions were written before the implementation. The first
run failed all four cases: multi-question composition was pre-empted by the
capabilities handler, Russian renaming and target-less modification returned
`unknown`, and free-time small talk exposed one canned response. The exact
failure output is preserved in
[`reproduction-before.log`](raw-data/reproduction-before.log).

The fix keeps the behavior data-led:

- decomposition prefers multiple actionable question segments and composes
  independently solved answers in source order;
- assistant-name set/read surfaces and the single clarification response are
  localized seed records;
- ambiguous target-less modification is a role/predicate that asks exactly one
  question before unknown fallback;
- response records can carry deterministic variants selected by stable prompt
  hashing, so repeated prompts are stable while the class is not canned.

The native green run is preserved in
[`reproduction-after.log`](raw-data/reproduction-after.log). The same four
contracts pass through the production browser worker in
[`issue-710.spec.js`](../../../tests/e2e/tests/issue-710.spec.js), while
[`issue-710-worker-parity.mjs`](../../../experiments/issue-710-worker-parity.mjs)
provides a fast worker-only probe.

## Smallest-leaf decomposition and self-hosting

The implementation was reviewed as five smallest independently verifiable
leaves:

1. native four-language reproduction specification;
2. native seed/parser/solver implementation;
3. browser-worker parity and real-browser regression suite;
4. 32-row evidence/status reconciliation and focused owners;
5. the verdict-definition contract artifact.

Formal AI, reached through the real external Agent CLI against the local
self-hosted release server, authored leaves 4 and 5. The drivers are
[`issue_710_agent_cli_audit_contract.sh`](../../../experiments/issue_710_agent_cli_audit_contract.sh)
and [`issue_710_agent_cli.sh`](../../../experiments/issue_710_agent_cli.sh).
Their captured sessions and byte-compared artifacts live in
[`agent-cli-evidence/audit-contract/`](agent-cli-evidence/audit-contract/) and
[`agent-cli-evidence/verdict-contract/`](agent-cli-evidence/verdict-contract/).
That is two of five named smallest leaves (**40%**), while the other three are
honestly recorded as manually authored.
