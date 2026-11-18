CI missed calendar seed integrity and native provenance failures, while several fixtures retained obsolete source ownership or uncontrolled network inputs. Compound task parsing also splits conjunctions inside ASCII quotations. This draft repairs those boundaries and the recorded fixture, client-constraint and CI regressions. The complete issue remains unfinished; keep this PR draft and unmerged.

Fixes #1580

Changes:

- Preserve ASCII quoted passages during task decomposition in both JavaScript and Rust, honor escapes and word apostrophes, and retain adjacent CJK quotations. Add ten shared regression cases without changing the original 15 inputs, outputs or traces; regenerate worker, TypeScript, seed and translation artifacts through maintained tools.
- Preserve decoded native structural and vocabulary values, including literal quotes at either edge. Shared held-out vectors and actual task quote-pair assertions cover the calendar/currency panic.
- Enforce request-local output constraints, decode path-bound OpenCode Read results, preserve calendar clock spans, and avoid empty startup logs during writability checks.
- Give calendar addition a unique meaning, expand 21 packed alias rows into 55 scalar rows without losing spellings or order, and update consumers and generated mirrors. Retain the existing response intent and native calendar/currency CI coverage.
- Preserve seeded counting membership proofs alongside retrieval, observe the connector's actual workspace probe, trace agent mode through current CLI/configuration owners, and freeze live network input in the interpretation fixture. Existing semantic, negative and full-answer assertions remain intact.
- Preserve complete localized unknown responses when no opener pool exists. The terminal JavaScript path now consults declared answer-language metadata and records the actual detected language. Regenerate both affected TypeScript mirrors with the maintained translator. The native regression remains outside production source, preserving the committed WASM source, binary and manifest.
- Retain bounded compiler-cache diagnostics; repair change-detection failures, required-context reporting, source-test placement and source ownership fixtures. Preserve generated workflow operations, bindings and deployed bytes while reducing fast source checkout inputs. Refresh improved source limits and reviewed governed fingerprints through their maintained tools.
- Retain template comparisons, upstream reports, complete failed CI attempts and explicit verification limits. Main `015a683197c89e8f74375b097e8dd06a57dcea8f` is merged.

Latest reproduction and verification:

- Before the quotation repair, `Count "salt and pepper" and report the size` incorrectly becomes three tasks. The focused before run has **7 passes and 2 failures**. A separate direct diagnostic reproduces the adjacent CJK single-quote defect. Afterward, the **25-case fixture check**, **9 executor tests** and **15 adjacent factual-QA tests** pass. All original 15 fixture entries remain unchanged, including traces.
- Shared-data validation initially records three stale generated artifacts. Preserve all failures, run the maintained seed and translation writers, then pass the complete inventory: **86 executable gates pass, 52 explicit native/CI-context exclusions, zero failures**. Worker and **717-file TypeScript projections**, Rust 2024 formatting and whitespace checks pass. Projection verification still refuses 49 modules; native quotation execution remains unverified.
- All **23 hosted workflows pass on earlier input `a9783223`**. On quotation head `04a468f5`, **21/23 pass**: the native calendar/currency test panics because the Rust grammar reader removes decoded quote characters, while a Layered checkout fails with curl 28 before tests. Both complete workflow logs are retained. The native reader repair passes **2 + 5 + 9 JavaScript tests**, Rust 2024 formatting and **10 selected gates**; its native execution and new-head CI remain pending. Scope-gated success does not prove native acceptance.
- Reject the supplied single-passage context candidate: only **4/8 qualification controls pass** across 16 finite VM observations with no transport calls. It can mark a passage-plus-arithmetic request resolved while omitting the passage answer. Parent answer coverage, multi-passage binding and withheld-child obligations remain open; do not suppress child tests or make the request atomic.
- The preceding declared-language correction passes **8/8 focused cases** after two before failures. Complete localized unknown responses and declared language metadata remain covered; native Spanish execution and the definition route remain unfinished. Both failed attempts on the earlier `edffcb18` head are retained separately from current success.

Run focused files separately:

```sh
timeout 600 node --max-old-space-size=256 --test --test-concurrency=1 rust/tests/web/formalization-native-parity.test.mjs
timeout 600 node --max-old-space-size=256 --test --test-concurrency=1 rust/tests/web/shared-formalization-grammar.test.mjs
timeout 600 node --max-old-space-size=256 --test --test-concurrency=1 rust/tests/web/issue-1550-meta-executor.test.mjs
timeout 600 node --max-old-space-size=256 --test --test-concurrency=1 rust/tests/web/issue-1172-factual-qa-subject-gate.test.mjs
node --max-old-space-size=256 experiments/issue-1550/parity-fixture.mjs --check
node --max-old-space-size=256 experiments/formal_ai_subagent/local-gates.mjs
```

The [quotation checkpoint](https://github.com/link-assistant/formal-ai/blob/issue-1580-8d61a32a61e1/docs/case-studies/issue-1580/task-quotation-checkpoint.md), [declared-language checkpoint](https://github.com/link-assistant/formal-ai/blob/issue-1580-8d61a32a61e1/docs/case-studies/issue-1580/declared-language-policy-checkpoint.md), [native boundary checkpoint](https://github.com/link-assistant/formal-ai/blob/issue-1580-8d61a32a61e1/docs/case-studies/issue-1580/opener-native-boundary-checkpoint.md) and [localized opener checkpoint](https://github.com/link-assistant/formal-ai/blob/issue-1580-8d61a32a61e1/docs/case-studies/issue-1580/localized-opener-and-ci-checkpoint.md) retain source identities, complete failed logs and before/after evidence.

Earlier verification, recorded separately:

- Calendar/currency: **40/42 before and 43/43 after**, preserving original cases and adding six alias prompts. Health/cache: **24/24**, fast-profile: **15/15**, ledger: **4/4**, with explicit network policy and no actual network I/O. The fast inventory contains 243 files.
- The complete combined-source gate run passes **86 checks**, with **52 explicit native/CI-context exclusions** and a 256 MiB Node heap. A later full 138-entry inventory initially has five failures; all five pass on focused reruns. Both initial failures and corrected receipts remain available.
- Sparse checkout regression: **15/16 before and 16/16 after**; all **20 underlying fast source checks** pass against a materialized selection. Selected working-tree bytes fall from 732,862,187 to 413,103,005 with measured sources retained. All **23 workflows pass on earlier input `7e89122c`**, after one checkout-only retry. This does not prove transport causality.
- Staged projection preserves **52 operations, 104 bindings and deployed workflow bytes**. All **3 stage transition tests**, seven affected gates and pinned runner checks pass. Parsed YAML equality is preserved while returning the canonical workflow to 1,494 lines under its unchanged 1,500-line ceiling.

The [checkout checkpoint](https://github.com/link-assistant/formal-ai/blob/issue-1580-8d61a32a61e1/docs/case-studies/issue-1580/checkout-archive-checkpoint.md), [calendar/cache checkpoint](https://github.com/link-assistant/formal-ai/blob/issue-1580-8d61a32a61e1/docs/case-studies/issue-1580/calendar-seed-checkpoint.md), [runtime checkpoint](https://github.com/link-assistant/formal-ai/blob/issue-1580-8d61a32a61e1/docs/case-studies/issue-1580/runtime-fixture-checkpoint.md) and [native evidence checkpoint](https://github.com/link-assistant/formal-ai/blob/issue-1580-8d61a32a61e1/docs/case-studies/issue-1580/native-evidence-checkpoint.md) explain those results and their limits.

Unfinished acceptance criteria remain in the [work plan](https://github.com/link-assistant/formal-ai/blob/issue-1580-8d61a32a61e1/experiments/issue-1580/work-plan.md): **53 distinct main native panic headers and three budget terminations**, semantic regressions, benchmark floors, full comparison and applicability decisions for all eight templates, dependency migrations and warnings, historical cache/Docker and required-context reproduction, live delivery/model identity, and parent-context decomposition. All 12 failed native test-job logs are preserved; complete literal review is unfinished. Fresh native catalog payloads still disagree with the committed artifact, whose strict contracts remain intact.

Java historical literal review covers **78/254 continuous pages**, 70 complete files, one partial file and 4,631 literally reviewed source lines; pages 79–254 remain unread. Full semantic applicability for all templates remains unfinished. Source reading, upstream reports and skipped native jobs do not establish acceptance. Independent Formal AI reasoning acceptance remains **0/20**; supplied commands, patches, scripts and conclusions receive zero independent authorship credit.
