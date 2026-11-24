# Interface replay and focused native compilation

Published `53d8212b` completes all 23 workflows: 21 pass, while the [native pipeline](https://github.com/link-assistant/formal-ai/actions/runs/44613974902) and [Layered CI](https://github.com/link-assistant/formal-ai/actions/runs/44613974224) fail. All 86 collection payload hashes verify. Final JavaScript shards pass 2,904 cases with 17 skips; nested focused runs are not counted twice.

## Reproduction and repair

The native isolated step never executes its behavior tests. With `RUSTFLAGS=-Dwarnings`, `cargo test --manifest-path rust/Cargo.toml --locked --no-run --test reasoning_contracts` rejects the unchanged public `observe_append` helper hidden inside the focused target's private module. Native job 135877853426, lines 1671–1680 preserves the exact compiler error. The focused root now exposes the shared helper module publicly, retaining every helper, its assertions and warning enforcement. Later independent stages pass thirteen library checks, one unsupported-language check and four native HTTP checks. Those fourteen Rust passes do not establish execution of the uncompiled target.

Layered CI rejects the stale `ts/worker/formal_ai_worker_source_endpoints.ts` projection (full workflow lines 10005–10007). Regenerating this file with `node scripts/translate-es.mjs js/worker/formal_ai_worker_source_endpoints.js` restores the maintained transformation; `node scripts/translate-es.mjs --check` verifies the complete generated tree.

## Complete existing interface cases

A separate native target includes 77 existing CLI, prompt-pipe, PTY, TUI isolation, memory, routing-family and retired-handler cases, with all original included module bytes unchanged. Its entry point sits beside the integration modules so ordinary child-module lookup preserves the existing wrapper fixtures. All seventeen consumed root/source/captured-input paths select the native lane. Sparse checkout already includes the required documentation captures. Captured-input hashes are identity evidence; they do not imply full literal reading of the large captures.

The missing-admission regression fails before implementation (49 pass, one fail). Local failures also expose and preserve the incorrect path-based child lookup, a test-expression delimiter mistake, module ordering and one nonexistent verification filename. After repair, all 65 focused JavaScript cases, Rust formatting, ShellCheck, generated-source parity and seven affected gates pass. The existing mock-runner controls verify that both compilation and execution receive the interface target, alongside failure collection and isolated-process overlap. No Rust build runs locally.

The next native total is expected to be 365 Rust cases and four HTTP cases. This is an inventory, not an observed result; every newly included interface/Hive case and the earlier procedure repair still require hosted execution. The twenty-minute deadline, assertion profile and warnings remain enforced.

## Remaining acceptance

Recounting actual prior `cf22c611` passes against complete original-file identity adds four previously uncounted catalog/concept/retrieval cases to the strict union: 27/53 original panic contracts qualify; 26 and all three original budget criteria remain unfinished. Current compilation failure adds no replay credit.

Retained evidence lives in `experiments/issue-1580/evidence/ci/hive-profile-53d8212b/`, `evidence/ci/offline-catalog-cf22c611/original-native-replay-union.json` and `evidence/checks/native-interface-*`. Sixty-two prior-publication files are archived. Full template semantic applicability, the remaining original criteria, wider runtime/delivery/provenance requirements and complete PR review remain open. Independent accepted Formal AI reasoning remains 0/20. [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581) stays draft and unmerged.

The first local publication attempt stopped before pushing: the filename ratchet counted 91 abbreviated paths against a ceiling of 90. The new feedback artifact used an abbreviated pull-request filename; it was renamed to `pull-request-conversation.json` without changing the ceiling. The failed gate and publication receipt are retained under `native-interface-admission/filename-gate-first-attempt`.
