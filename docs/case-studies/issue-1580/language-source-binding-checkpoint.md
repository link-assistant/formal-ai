# Browser meaning hydration and native unresolved-language source binding

PR: https://github.com/link-assistant/formal-ai/pull/1581

The published input `7f19154a21d79f268f48af6b0938f17e6fb32ba3` completes **23/23 workflows successfully**. The actual native job is https://github.com/link-assistant/formal-ai/actions/runs/44320550638/job/134858261266. Its retained raw log has SHA-256 `9e5ee69e03209a27cd3db0f6fe34e0e04af78ce4edfc15459e7b6ea2a9269487`; selected lines 1630–1672 show calendar/currency 1/1, executor 8/8, forced functions 1/1 and synthesis 12/12. The full 1,855-line payload is retained, without a full literal-read claim. This is earlier published verification; the language changes below still need their own hosted execution.

## Browser root cause and regression

The generated `FORMAL_AI_CRATE_MEANING_SEEDS` registry includes `learned-request-openers` and `software-project-phrases`. Their text was fetched but the meaning hydrator accepted only `meanings.lino` and `meanings-*.lino`. It omitted the registered files, including the Spanish definition and Google Trends request frames. It also retained a warmed `WEB_SEARCH_MARKERS_CACHE` after replacing seed text.

The hydrator now accepts the exact registered `.lino` basenames as well as the existing extension-fixture convention, and invalidates the derived marker cache. No production prompt or language-specific phrase is added to JavaScript. Tests reject an unregistered basename and a wrong extension, replace warmed markers, and replay **both unchanged prompts** from the authored native Spanish sibling contract. The full native Spanish “how” answer remains an exact equality assertion.

With fixture text corrected to the existing space normalization, the unchanged production worker fails three of ten focused tests: the registered Spanish request, registered synthetic frame and warmed-marker replacement. After the production fix, **10/10 pass**. The earlier first attempt also had a hyphen normalization mistake in test expectations; its failing log is retained separately and does not establish an additional production defect.

Raw focused log hashes:

- Initial fixture draft: `65a7d0dcfdd103580ecce1c8dde2e2c856dc5069023efc21a144008e0fadae1e`.
- Valid pre-fix regression: `82b87ed021d42f7cb55b0642790125753fd98a0dd3d4e55407cf468b67bb844e`.
- Passing post-fix regression: `9a1184fb04e02a133cbe91c7e1be61d7919ab0d50de795b24705364e7f20c051`.

The four adjacent files pass (boot prefetch 1/1, transport 2/2, unresolved routing 5/5 and fixture integrity 2/2), as do the five selected worker/specification gates. A first foreground batch exceeded the Formal AI shell tool's own 60-second limit during the last gate. Its completed checks were preserved, and only the unfinished gate resumed in a background worker; all nine checks subsequently completed. No local native build runs.

## Real browser evidence

Run the bounded local server `python3 experiments/issue-1580/browser-meaning-server.py`, then the supplied `capture-meaning-hydration-browser.js` through Playwright MCP. The server substitutes only the exact published meaning hydrator on port 8048 and the working hydrator on 8049. Each server has a ten-minute deadline and 2,000-request limit. The harness sends both original Spanish prompts to the real production Worker with `jsfallback=1`. The Playwright context blocks external hosts; no native/WASM execution is claimed. The optional search-component bundle is absent, so its maintained unavailable-component fallback is used.

The generated production app UI bundles are absent from this checkout. These screenshots deliberately show a labeled regression harness, **not the production app UI**. Chromium reports `[unknown, unknown]` before and `[web_search, unknown]` after. The complete second answer is byte-equal between phases. Full browser observations, response evidence, screenshot hashes, bounded server logs and receipts are retained under `experiments/issue-1580/evidence/checks/browser-meaning-hydration/browser/`. The browser is closed and both server ports are confirmed closed.

Before:

![Published meaning hydration in the production worker regression harness](https://github.com/link-assistant/formal-ai/blob/issue-1580-8d61a32a61e1/docs/screenshots/issue-1580/meaning-hydration-before.png?raw=true)

After:

![Registered meaning hydration in the production worker regression harness](https://github.com/link-assistant/formal-ai/blob/issue-1580-8d61a32a61e1/docs/screenshots/issue-1580/meaning-hydration-after.png?raw=true)

## Native source binding

The retained original main failure returned `concept_lookup` for `¿Cómo funciona la fotosíntesis submarina de xyzzy?` instead of the authored `unknown` response. `answer_unknown_prompt` tried retrieved senses before its unsupported-language fallback; the highest-tier retrieved word gloss could replace an entire unparsed sentence.

The narrow guard refuses retrieved glosses as answers to non-bare prompts in languages outside the declared answer set, retaining an `external_source:no_verified_answer` gather result. Existing supported-language definitions and unsupported-language bare terms remain eligible for attributed concept lookup. This does not claim generic answer binding for all supported-language questions, memory/cache matches or multi-passage reasoning.

Four tests use the real committed `issue-1138-b1/expected-senses.json` capture metadata. They check the exact original full Spanish answer against an unrelated retrieved word, a word actually mentioned in an unparsed question, an attributed English definition, and an attributed Spanish bare term. Hosted CI will execute these library tests and the **unchanged original** `a_language_without_localized_openers_reports_a_gap_not_english` contract as an isolated `any_language` target. The admission classifier includes both test inputs and their embedded fixture. JavaScript policy tests inspect the real workflow and source closure.

The first local static checks exposed formatting, the old exact workflow-command list and the governed Cargo fixture hash. All three were corrected with full logs retained. The reviewed witness refresh updates the four references to `rust/Cargo.toml` and asserts complete-object equality after only those hash replacements. All 1,185 records (221 supported, 964 unsupported), the authoritative native source manifest and every unsupported object are retained. The original reader is unchanged. A subsequent main integration and its complete witness delta require separate review before publication.

## Reviewed main integration

The integration with main `30ff2d8a81cf8f21f679c2826851b5b413c34b7a` preserves both branches' behavior. The isolated native regression step now includes the existing four targets plus main's `cnl_output` and `cnl_parity`. The four captured-source units and unchanged Spanish sibling remain a separate step. The CNL parity runner discovers fixtures at runtime, so changes anywhere under `data/parity/issue-1584-cnl/` select native execution, including future language fixtures. The real workflow policy has 24 passing tests. This closes the merge's admission gap without a local Rust build.

Both TypeScript/JavaScript pairs preserve main's controlled-language answer projection and this branch's same-language and meta-subtask guards. The native solver retains main's controlled solve closure with this branch's validated intent selection. Generated worker prerequisite and runtime-feature identities are regenerated from the resolved sources.

The static native program registry is regenerated through its maintained JavaScript writer. The complete comparison preserves all 157 prior programs from each parent and adds `formal_ai::cnl_output::CNL_POLICY_PATH`. Against this branch, only the 157 library source-witness hashes change. Main's registry already contains 16 stale hashes across `derivation.rs`, `memory_sync.rs` and `service_accessibility.rs`, plus a stale `LEARNED_CHUNK_KIND` source span. The merged registry repairs those identities and offsets; its current 9676–9734 span selects the exact unchanged native constant declaration. Every current source-witness hash is checked against actual bytes. All other complete program fields remain equal. The initial stricter review failures are retained rather than treated as production failures.

The merged translation ledger uses all 164 committed source projections: 342 translated, 38 workaround, three inferred and 2,656 carried records; 114 modules have translations, one is fully translated and 50 are refused. The maintained renderers and regression ratchets against both parent ledgers pass. No native translation build, projection replacement or reduced threshold is used.

The governed witness is compared as complete objects against both parents before the maintained explicit writer refreshes it. Against this branch, 111 source-hash references change and one unchanged unsupported OpenAI compatibility test moves into its new `output.rs` module. Against main, 26 source-hash references change. Every previous/current source hash is tied to its exact Git blob/current file. All 1,185 records remain: 221 supported and 964 unsupported. The original reader remains unchanged. The complete delta, both parent binary archives, reviewed candidate and writer receipt are retained under `experiments/issue-1580/evidence/checks/language-main-integration/`.

During conflict resolution, the ordinary Formal AI driver could not import conflict-marked prerequisite modules. A temporary coherent snapshot of local commit `a748854197dcc693033610acc6589f10b1c6d525` was used only to execute the supplied merge-resolution commands. Host snapshot construction and the failed preload attempt receive zero solver or validation credit. No identity check was disabled. All merged-tree checks run through the restored ordinary driver against the actual checkout, without the preload or snapshot.

## Local verification and pending hosted execution

The first pre-merge inventory completes in 764.618 seconds: 80 executable gates pass, six fail and 52 are explicitly excluded. All six failures trace to placing the test-only module under `rust/src`. The module has moved to the existing test tree without changing any ceiling or allowlist. All 91 initial aggregate/focused/detailed payload hashes are verified.

On the resolved merged tree, Rust formatting and all ten focused JavaScript files pass: workflow policy 24/24, browser language 10/10, fixture integrity 2/2, governed identifiers 34/34, request context 4/4, CNL output 13/13, HTTP 4/4, dialogue 7/7, parity 6/6 and grammar matrix 8/8. Their complete logs have been read and hash-verified. The first merged shared inventory reports 81 passes, five failures and 52 explicit exclusions. The failures expose an oversized structured evidence file, a duplicated generated status row, and missing exact answer examples in three new native tests. Complete evidence objects are retained in compact JSON, the maintained status writer regenerates the row, and exact answers are added while preserving the existing assertions. All five affected gates pass on retry. The next complete inventory reports **85 passes, one failure and 52 explicit native/context exclusions**. Its remaining failure identifies four preflight payload filenames using `pr`; the payloads are renamed to `pull-request` with byte equality verified, and the unchanged abbreviation gate passes on retry. Thus all 86 executable gates have passing results after their affected corrections; this is not a claim that the last full invocation had zero failures. Rust formatting and whitespace pass. All 89 complete-inventory payload hashes, the filename retry payload and candidate source identities are verified. No source changes during that run and no local Rust build runs.

The four new native tests, unchanged Spanish sibling and merged six-target execution remain unverified until they run on the actual published head. Every broader issue criterion remains unfinished in the work plan. Independently accepted Formal AI reasoning remains **0/20** planned substantive units, below the standing 5% reporting target. Supplied patches, scripts, commands and review earn zero independent authorship credit. The standing doctrine R1017-R5 makes this a reporting target, without adding a release percentage gate; the broader unfinished issue requirements still prevent completion. PR #1581 remains draft and unmerged.
