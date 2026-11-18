# Localized opener and CI metadata checkpoint

This is unfinished work for [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581).
The PR remains draft. The broader issue acceptance criteria and independent
Formal AI reasoning acceptance (0/20) remain open. Supplied commands, patches and
scripts earn zero independent authorship credit.

## Published-head failures

All 23 workflows on `b367d603375dd23223861c92875e079a3a90df13` have finished:
21 pass and two fail. The failures are introduced bookkeeping misses; the earlier
green head does not establish that this head passes.

| Run | Complete primary failure evidence | Correction |
| --- | --- | --- |
| [Pipeline 44220008975](https://github.com/link-assistant/formal-ai/actions/runs/44220008975) | Job 134519246760 lines 751–754 reports a 795→793 line improvement and a 36,519→36,517 core total. The status job only propagates this failure. | Lower both ledger limits and regenerate the maintained status surface. No migration disposition is changed. |
| [Layered CI 44220008509](https://github.com/link-assistant/formal-ai/actions/runs/44220008509) | Jobs 134519390942 lines 4860–4866 and 134519390973 lines 2754–2768 reject the stale `formalization.rs` source hash in the governed witness. | Compare every complete case and unsupported-case object, then use the maintained reviewed-refresh command. Only one source-manifest hash changes. |

Complete aggregate and failed-job logs, identities and fresh run snapshots are
retained under `experiments/issue-1580/evidence/ci/native-b367d603/`.
The complete primary failure payloads have been read; full literal review of
every line of all aggregate logs is not claimed. Neither failing suite exceeds
its time budget, and no CI retry is used for these failures.

The witness review preserves all 221 supported and 964 unsupported complete
objects, the manifest paths and the reader identity. Its sole hash delta is the
previously published offline interpretation fixture. The complete comparison,
old witness and maintained refresh receipt are under
`evidence/checks/localized-unknown-opener/governed-refresh/`. This is a reviewed
source-fingerprint refresh, not a change to test expectations.

## Preserve localized unknown responses

The opener registry has no Spanish pool and explicitly falls back to English
when callers ask for the pool itself. Previously, variation selection reused
that fallback and replaced the first sentence of a Spanish seed response with
an English opener. Both selectors now require a pool for the requested language.
The native formatter keeps the complete seed response when selection returns an
empty opener. The existing pool-query fallback API remains unchanged.

Reproduce the JavaScript regression with:

```sh
timeout 600 node --max-old-space-size=256 --test --test-concurrency=1 rust/tests/web/worker-unknown-language.test.mjs
```

The unchanged original Spanish prompt produces two failing new checks before
the fix; all six checks pass afterward. They require full Spanish seed equality,
an empty selector result for the missing pool, the original English pool-query
fallback and a valid English selection. The four existing cases still pass.
The native unit regression uses a generic probe input to check the same
missing-pool behavior without embedding a held test prompt in production source.
That native test has not been executed locally.

The initial full gate run reports 133 passes or skips out of 138 entries, with
five failures. The new native test's literal prompt trips two prose-policy
checks; the generated status page is stale; ShellCheck is missing. Replace only
the native test input with a generic probe, regenerate status with the maintained
renderer, and extract the official ShellCheck v0.11.0 release to scratch storage.
The archive SHA-256 matches GitHub's asset digest. All five failed entries pass
on focused reruns. The repository gate definitions and shell lint command stay
unchanged. The initial failed logs are retained alongside the corrected results.

The two CI-failing JavaScript files pass separately (2 and 34 cases), the six
affected policy/core/status gates pass, and Rust formatting, JavaScript-to-Rust
projection verification and whitespace checks pass. These checks compile no Rust.
The projection report still has 49 refused modules; its successful check is not
complete language parity. Logs and final source identities are under
`experiments/issue-1580/evidence/checks/localized-unknown-opener/`.

## Remaining acceptance

The original native Spanish routing and complete unresolved-reasoning contracts
remain open. The retained pre-fix production-worker probe records their current
JavaScript routing and seed-answer gap, with attempted fetches refused and no
actual network traffic. A separate after-fix replay preserves that original
evidence and shows Spanish openers for both unchanged native sibling inputs;
both still route to `unknown`. Fixing the English opener does not establish
either native routing or a complete reasoning trace.

The 53 recovered native panic headers, three native budget terminations,
benchmark floors, warnings, catalog input determinism, all eight complete
template reviews, recursive context binding, dependency migration and live
serving/delivery acceptance remain unfinished. No local native build, broad
native behavioral pass, ready transition or merge is claimed.

Final publication checks pass six of seven entries, then catch two abbreviated
evidence filenames added during review. Renaming those files in full words
restores the unchanged 90-file naming ceiling, and the naming/whitespace rerun
passes. The initial failed publication log and the correction receipt are both
retained under `evidence/checks/localized-unknown-opener/publication/`.

## Subsequent published-head correction

The next head `d0b395bb` completed with 22 passing workflows and one failure.
The [native boundary checkpoint](opener-native-boundary-checkpoint.md) records
both actual failures, restores the recorded WASM input and moves the native
test out of production source. Its formatter guard supersedes the shared-core
selector change described above; the JavaScript selector guard remains in place.
