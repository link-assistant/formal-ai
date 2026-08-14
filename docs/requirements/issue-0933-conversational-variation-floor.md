## Issue #933 Conversational Wording-Variation Floor

Issue [#933](https://github.com/link-assistant/formal-ai/issues/933) is the CI
enforcement that issue
[#123](https://github.com/link-assistant/formal-ai/issues/123) asked for and
PR [#124](https://github.com/link-assistant/formal-ai/pull/124) did not deliver:
every conversational test case must hold at least five distinct wording
variations in each of `en`, `ru`, `hi` and `zh`. The corpus is data
(`data/benchmarks/conversational-variations-suite.lino` plus its per-language
partitions), the count is a CI gate
(`tests/e2e/scripts/check-conversational-variation-floor.mjs`), and every
recorded prompt is answered by the real engine
(`tests/unit/conversational_variations.rs`). Evidence, the "before" measurement
and the dedup analysis against the existing language checks are in
[`docs/case-studies/issue-933/`](../case-studies/issue-933/README.md).

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R933-1 | "Wording variation" is machine-checkable, not a convention in comments. | Implemented: a manifest declares the cases, the languages and `minimum_variations_per_language`, and each prompt is a `conversational_variation_case` record carrying its case, language, expected intent and expected evidence link. Two prompts count as one wording unless they differ in more than case, punctuation, symbols or spacing — the Node check and the Rust runner implement that rule identically, so the two halves cannot disagree. |
| R933-2 | A CI script walks the corpus and fails if any test case holds fewer than five variations in any advertised language. | Implemented: `npm run --prefix tests/e2e check:variation-floor`. It also rejects duplicate ids, unlisted cases or languages, records in the wrong partition, prompts that are empty once punctuation is stripped, and prompts that only re-punctuate one already counted. |
| R933-3 | Under-covered cases are backfilled until the check passes. | Implemented: 10 cases × 4 languages, 228 prompts, smallest group 5. The router phrasings the backfill needed were added to `data/seed/intent-routing.lino` and `data/seed/meanings-intent.lino`; 8 of 24 groups in the previous corpus were below the floor (`docs/case-studies/issue-933/raw-data/legacy-counts-before.txt`). |
| R933-4 | The check runs in CI. | Implemented: `data/meta/ci-gates/check-conversational-variation-floor.lino`, stage `web`, run by `rust-script scripts/run-ci-gates.rs --stage web` from `.github/workflows/release.yml` (issue #991 replaced the workflow's step list with this registry). |
| R933-5 | A unit test covers the counting logic with fixture data engineered to trip the floor. | Implemented: `tests/web/conversational-variation-floor.test.mjs` — a group at four, a language with no records at all, and five re-punctuated copies of one phrase that must count as one. |
| R933-6 | Every recorded variation is one the engine actually routes. | Implemented: `tests/unit/conversational_variations.rs` answers all 228 prompts with `FormalAiEngine` and asserts intent, evidence link and, for arithmetic, the answer text. Counting alone cannot be satisfied with wordings the router rejects. |
| R933-7 | Coverage counts print per language, and a failure names exactly which cases are under the floor. | Implemented: the count table prints on success as well as failure; each shortfall prints `- case <name> has <n> <language> variation(s); the floor is 5` followed by the file to edit. |
| R933-8 | The new gate does not duplicate `check:language-test-coverage`. | Confirmed: that gate is diff-aware and satisfied by one mention of each registered language in a pull request's added test lines; it can never observe a case stuck at four wordings. Full comparison against it, `check:language-change-parity` and `check:intent-coverage` in the case study. |
