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

## 2026-08-01 re-verification

The checklist below was re-run against current `main` rather than inferred from
old issue closure state. The allowed verdict vocabulary is deliberately closed:
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
| 7 | Chat | Folder-listing prompt variants | `superseded` | General capability routing in [#745](https://github.com/link-assistant/formal-ai/issues/745), [#758](https://github.com/link-assistant/formal-ai/issues/758), and merged [PR #850](https://github.com/link-assistant/formal-ai/pull/850); pinned by [`issue_745.rs`](../../../tests/unit/issue_745.rs), `directory_listing_routes_shell_variations_in_every_supported_language`. |
| 8 | Chat | Target-less modification asks one question | `works-now` | [`issue_710.rs`](../../../tests/unit/specification/issue_710.rs), `ambiguous_modifications_ask_exactly_one_question_in_every_language`, with the same worker assertion in [`issue-710.spec.js`](../../../tests/e2e/tests/issue-710.spec.js). |
| 9 | Chat | Multiple deterministic free-time replies | `works-now` | [`issue_710.rs`](../../../tests/unit/specification/issue_710.rs), `free_time_answers_are_prompt_stable_but_not_one_canned_reply`; variants are seed records rather than runtime randomness. |
| 10 | Chat | Assistant name set/read and attribution | `works-now` | [`issue_710.rs`](../../../tests/unit/specification/issue_710.rs), `assistant_name_can_be_set_and_recalled_in_every_language`; attribution remains pinned by [`issue-157.spec.js`](../../../tests/e2e/tests/issue-157.spec.js). |
| 11 | Localization | Issue #292 rules, answer-language, parity, and Markdown asks | `works-now` | [`behavior_rules.rs`](../../../tests/unit/specification/behavior_rules.rs), `behavior_rules_list_answer_is_localized_for_supported_languages`; [`multilingual.spec.js`](../../../tests/e2e/tests/multilingual.spec.js), `reported Russian behavior-rule list is localized and markdown-safe`; generated parity scripts live under [`tests/e2e/scripts`](../../../tests/e2e/scripts/). |
| 12 | Localization | Thinking localization outside the browser UI | `still-broken` | Focused owner [#889](https://github.com/link-assistant/formal-ai/issues/889); `src/thinking.rs` still emits English detail consumed by CLI/API/Telegram. |
| 13 | Localization | Collapsed thinking animation and top placement | `works-now` | [`issue-488.spec.js`](../../../tests/e2e/tests/issue-488.spec.js), `shows collapsed human-readable thinking by default and expands details` and `localizes thinking preview and detail settings across supported languages`; narrative order is pinned in [`issue_676_thinking_narrative.rs`](../../../tests/unit/issue_676_thinking_narrative.rs). |
| 14 | Knowledge | Translate formal proofs to programming languages | `still-broken` | Focused owner [#890](https://github.com/link-assistant/formal-ai/issues/890); [`issue_403.rs`](../../../tests/unit/issue_403.rs) proves solving only, not proof translation. |
| 15 | Knowledge | At least 50 verified equation types | `still-broken` | Focused corpus/ratchet owner [#891](https://github.com/link-assistant/formal-ai/issues/891); the existing equation regressions cover fewer than 50 cases. |
| 16 | Knowledge | Compose calculations with other instructions | `works-now` | [`calculator_delegation.rs`](../../../tests/unit/specification/calculator_delegation.rs), `embedded_request_variations`, compound-interest continuation tests, and the issue-710 independent-question specification. |
| 17 | Knowledge | Word problems beyond train meeting | `works-now` | [`calculator_delegation.rs`](../../../tests/unit/specification/calculator_delegation.rs), `fibonacci_word_problem_reduces_to_calculator_expression` and `box_relation_word_problem_resolves_total_with_reasoning`. |
| 18 | Knowledge | Current films in release order, not a stale seed | `still-broken` | Focused source-backed replacement [#892](https://github.com/link-assistant/formal-ai/issues/892); [`issue_462.rs`](../../../tests/unit/specification/issue_462.rs) exposes the fixed snapshot ending in 2023. |
| 19 | Knowledge | Closest contextual pronoun resolution | `works-now` | [`issue_465.rs`](../../../tests/unit/specification/issue_465.rs), `pronoun_followup_resolves_prior_rust_topic_for_creator_question`, with multilingual fact availability. |
| 20 | Knowledge | How-to multi-source synthesis and seven-day availability cache | `still-broken` | Existing focused owner [#709](https://github.com/link-assistant/formal-ai/issues/709); the current exact-capture summarizer does not close this service-availability contract. |
| 21 | Knowledge | Iterative two-file summary validation and 80% quality bar | `still-broken` | Focused owner [#893](https://github.com/link-assistant/formal-ai/issues/893); recursive embedded grammars work, but the sampling/quality ratchet is absent. |
| 22 | Knowledge | Interior/plain-capitalized entity reasoning class | `works-now` | [`issue_571.rs`](../../../tests/unit/issue_571.rs), `external_entity_questions_route_to_web_search_by_reasoning_not_vocabulary`; lower-case Tesla routing remains pinned in [`multilingual.spec.js`](../../../tests/e2e/tests/multilingual.spec.js). |
| 23 | Platform | Calendar interchange and Apple/Google/Microsoft flows | `works-now` | [`calendar_ics.rs`](../../../src/solver_handlers/calendar_ics.rs) emits RFC 5545 accepted by Apple Calendar, Outlook, and Google Calendar plus a Google insertion URL; [`issue-404.spec.js`](../../../tests/e2e/tests/issue-404.spec.js) exercises the production worker. |
| 24 | Platform | Optional gated OCR and image transcription | `works-now` | [`issue-493.spec.js`](../../../tests/e2e/tests/issue-493.spec.js), `uses OCR text to flag the false ETH 2024 price claim`, plus multilingual/generalization coverage and the explicit OCR preference gate. |
| 25 | Platform | E2E against deployed GitHub Pages | `works-now` | [`workflow_release.rs`](../../../tests/unit/ci-cd/workflow_release.rs), `pages_e2e_uses_deployment_output_url` and `pages_deploy_is_pinned_and_live_e2e_waits_for_matching_deployment`. |
| 26 | Platform | Four-template CI comparison and upstream filings | `still-broken` | The comparison is preserved in [`REPORT.md`](../issue-479/template-comparison/REPORT.md); focused revalidation/filing owner [#894](https://github.com/link-assistant/formal-ai/issues/894) tracks the links that were left ready-to-file. |
| 27 | Platform | Published coverage with a non-decreasing ratchet | `still-broken` | CI retains LCOV and checks configured upload failure in [`issue_717.rs`](../../../tests/unit/ci-cd/issue_717.rs), but threshold enforcement is owned by [#895](https://github.com/link-assistant/formal-ai/issues/895). |
| 28 | Platform | Gemini headless tools | `works-now` | Superseding all-client work [#671](https://github.com/link-assistant/formal-ai/issues/671) / [PR #814](https://github.com/link-assistant/formal-ai/pull/814) records Gemini headless `read_file` requests and tool calls in [`recorded/gemini/read-file.jsonl`](../../../experiments/agentic_cli_matrix/recorded/gemini/read-file.jsonl). |
| 29 | Platform | macOS signed/notarized auto-update production path | `works-now` | [`issue-548.spec.js`](../../../tests/e2e/tests/issue-548.spec.js) pins version/event/localization behavior; [`desktop-release.yml`](../../../.github/workflows/desktop-release.yml) owns signing, notarization, update metadata, and explicit ad-hoc fallback diagnostics. |
| 30 | Platform | link-foundation/start and command-stream adoption | `still-broken` | Existing focused owners [#8](https://github.com/link-assistant/formal-ai/issues/8) and [#195](https://github.com/link-assistant/formal-ai/issues/195); neither package is the production command-execution boundary yet. |
| 31 | Platform | web-search/web-capture as real components | `still-broken` | Focused integration owner [#896](https://github.com/link-assistant/formal-ai/issues/896); current internal adapters do not depend on either published component. |
| 32 | Platform | Iframe pre-check and external-link actions | `works-now` | [`multilingual.spec.js`](../../../tests/e2e/tests/multilingual.spec.js), `GitHub navigation suggests an external link without iframe preview` and `Navigation previews URLs when frame policy allows embedding`. |

Totals: **21 `works-now`**, **10 `still-broken`**, **1 `superseded`**, and
**0 `blocked-upstream`**. The result is therefore partial by design: the audit
is complete and every gap has an owner, but the ten open feature gaps are not
misrepresented as implemented.

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
self-hosted release server, authored leaf 5. The driver is
[`issue_710_agent_cli.sh`](../../../experiments/issue_710_agent_cli.sh); the
captured session and byte-compared artifact live in
[`agent-cli-evidence/verdict-contract/`](agent-cli-evidence/verdict-contract/).
That is one of five named smallest leaves (**20%**), while the other four are
honestly recorded as manually authored.
