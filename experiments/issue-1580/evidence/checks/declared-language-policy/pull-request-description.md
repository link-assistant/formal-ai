CI missed calendar seed integrity and native provenance failures, while several fixtures retained obsolete source ownership or uncontrolled network inputs. This draft repairs those failures together with client constraints and CI regressions. The complete issue remains unfinished; keep this PR draft and unmerged.

Fixes #1580

Changes:

- Enforce request-local output constraints, decode path-bound OpenCode Read results, preserve calendar clock spans, and avoid empty startup logs during writability checks.
- Give calendar addition a unique meaning, expand 21 packed alias rows into 55 scalar rows without losing spellings or order, and update consumers and generated mirrors. Retain the existing response intent and native calendar/currency CI coverage.
- Preserve seeded counting membership proofs alongside retrieval, observe the connector's actual workspace probe, trace agent mode through current CLI/configuration owners, and freeze live network input in the interpretation fixture. Existing semantic, negative and full-answer assertions remain intact.
- Preserve complete localized unknown responses when no opener pool exists. The terminal JavaScript path now consults declared answer-language metadata and records the actual detected language. Regenerate both affected TypeScript mirrors with the maintained translator. The native regression remains outside production source, preserving the committed WASM source, binary and manifest.
- Retain bounded compiler-cache diagnostics; repair change-detection failures, required-context reporting, source-test placement and source ownership fixtures. Preserve generated workflow operations, bindings and deployed bytes while reducing fast source checkout inputs. Refresh improved source limits and reviewed governed fingerprints through their maintained tools.
- Retain template comparisons, upstream reports, complete failed CI attempts and explicit verification limits. Main `015a683197c89e8f74375b097e8dd06a57dcea8f` is merged.

Latest reproduction and verification:

- The original native Spanish how-question fixture supplies the complete expected JavaScript response. Before the declared-language correction, six cases pass and the full-response and held-out metadata controls fail. Afterward, **8/8 focused cases pass**, including Persian, Arabic, greeting, empty-input, unverified-memory and opener controls. The adopted Spanish definition route and native execution remain unfinished.
- Preserve the initial worker-budget failure, restore the unchanged **33-line ceiling**, and pass all **six affected source gates**. All **five final release, naming and size gates**, the **717-file TypeScript projection** and whitespace checks pass. No shared `data/meta` input changes this round. Projection verification still refuses 49 modules and does not establish full language parity.
- Published input `edffcb18` has **22 passing workflows and one failing Layered CI workflow**. Attempt 1 fails checkout before application tests; all 171 direct job-log lines are read. One bounded failed-job retry passes checkout and both JavaScript groups, then fails on a stale TypeScript opener mirror; all 320 direct failed job-log lines are read. The maintained translator reproduces and corrects that drift and the new terminal-policy mirror. Both complete attempts are retained. Checkout transport cause remains unverified; the next head still needs hosted verification.
- The preceding native-boundary correction passes the two CI-failing JavaScript files and localized-worker file separately (**5 + 5 + 6 cases**), all **12 touched gates**, formatting and projection checks. It restores the recorded WASM input byte for byte. No local native build or application execution is claimed.

Run focused files separately:

```sh
timeout 600 node --max-old-space-size=256 --test --test-concurrency=1 rust/tests/web/worker-unknown-language.test.mjs
timeout 600 node --max-old-space-size=256 --test --test-concurrency=1 rust/tests/web/rust-specification-fixture-integrity.test.mjs
timeout 600 node --max-old-space-size=256 --test --test-concurrency=1 rust/tests/web/rust-specification-unicode-identifiers.test.mjs
node --max-old-space-size=256 scripts/translate-es.mjs --check
```

The [declared-language checkpoint](https://github.com/link-assistant/formal-ai/blob/issue-1580-8d61a32a61e1/docs/case-studies/issue-1580/declared-language-policy-checkpoint.md), [native boundary checkpoint](https://github.com/link-assistant/formal-ai/blob/issue-1580-8d61a32a61e1/docs/case-studies/issue-1580/opener-native-boundary-checkpoint.md) and [localized opener checkpoint](https://github.com/link-assistant/formal-ai/blob/issue-1580-8d61a32a61e1/docs/case-studies/issue-1580/localized-opener-and-ci-checkpoint.md) retain exact source identities, complete logs and before/after failures.

Earlier verification, recorded separately:

- Calendar/currency: **40/42 before and 43/43 after**, preserving original cases and adding six alias prompts. Health/cache: **24/24**, fast-profile: **15/15**, ledger: **4/4**, with explicit network policy and no actual network I/O. The fast inventory contains 243 files.
- The complete combined-source gate run passes **86 checks**, with **52 explicit native/CI-context exclusions** and a 256 MiB Node heap. A later full 138-entry inventory initially has five failures; all five pass on focused reruns. Both initial failures and corrected receipts remain available.
- Sparse checkout regression: **15/16 before and 16/16 after**; all **20 underlying fast source checks** pass against a materialized selection. Selected working-tree bytes fall from 732,862,187 to 413,103,005 with measured sources retained. All **23 workflows pass on earlier input `7e89122c`**, after one checkout-only retry. This does not prove transport causality.
- Staged projection preserves **52 operations, 104 bindings and deployed workflow bytes**. All **3 stage transition tests**, seven affected gates and pinned runner checks pass. Parsed YAML equality is preserved while returning the canonical workflow to 1,494 lines under its unchanged 1,500-line ceiling.

The [checkout checkpoint](https://github.com/link-assistant/formal-ai/blob/issue-1580-8d61a32a61e1/docs/case-studies/issue-1580/checkout-archive-checkpoint.md), [calendar/cache checkpoint](https://github.com/link-assistant/formal-ai/blob/issue-1580-8d61a32a61e1/docs/case-studies/issue-1580/calendar-seed-checkpoint.md), [runtime checkpoint](https://github.com/link-assistant/formal-ai/blob/issue-1580-8d61a32a61e1/docs/case-studies/issue-1580/runtime-fixture-checkpoint.md) and [native evidence checkpoint](https://github.com/link-assistant/formal-ai/blob/issue-1580-8d61a32a61e1/docs/case-studies/issue-1580/native-evidence-checkpoint.md) explain those results and their limits.

Unfinished acceptance criteria remain in the [work plan](https://github.com/link-assistant/formal-ai/blob/issue-1580-8d61a32a61e1/experiments/issue-1580/work-plan.md): **53 distinct main native panic headers and three budget terminations**, semantic regressions, benchmark floors, full comparison and applicability decisions for all eight templates, dependency migrations and warnings, historical cache/Docker and required-context reproduction, live delivery/model identity, and parent-context decomposition. All 12 failed native test-job logs are preserved; complete literal review is unfinished. Fresh native catalog payloads still disagree with the committed artifact, whose strict contracts remain intact.

Java historical literal review covers **58/254 pages** (1–41 and 48–64), 65 complete files, two partial files and 3,038 complete source lines; pages 42–47 and 65–254 remain unread. Source reading, upstream reports and skipped native jobs do not establish acceptance. Independent Formal AI reasoning acceptance remains **0/20**; supplied commands, patches, scripts and conclusions receive zero independent authorship credit.
