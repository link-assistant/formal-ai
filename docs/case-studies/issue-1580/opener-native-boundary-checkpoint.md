# Localized opener boundary correction

[PR #1581](https://github.com/link-assistant/formal-ai/pull/1581) remains draft.
This checkpoint does not complete issue #1580 or establish native behavior.
Supplied patches and scripts earn zero independent authorship credit; the
independent reasoning acceptance remains 0/20.

All 23 workflows on `d0b395bb2310e285c89d93818571e4d54ed59c02` finished:
22 passed and [Layered CI 44226415780](https://github.com/link-assistant/formal-ai/actions/runs/44226415780)
failed. Its only failing job, 134540701574, has two failed tests out of
1,263 cases: 1,257 pass and four skip. It completes in 173 seconds of its
360-second execution budget. No timeout or retry is involved.

The complete aggregate log, paginated job inventory and direct failed-job log
are preserved under `experiments/issue-1580/evidence/ci/localized-opener-d0b395bb/`.
The complete primary failure payload, direct job lines 4846–4900, has been read.
Literal review of every line of both complete logs is not claimed.

| Failure | Cause | Correction |
| --- | --- | --- |
| `issue-1569-wasm-worker-inputs.test.mjs`, lines 4857–4875 | The prior patch changed `web_engine_core.rs`, an input of the committed WASM worker. | Restore that file byte for byte to its recorded input. Keep the localized language guard in the native answer formatter and existing JavaScript wrapper. The worker binary and manifest are unchanged. |
| `source-test-placement-twin.test.mjs`, lines 4877–4897 | The prior patch placed a unit test inline under production `src/`. | Register an external test module under `rust/tests/source/`, retaining the original full-seed assertion and adding controls for every authored language pool. |

The native formatter reads the canonical opener registry with the existing
Links Notation parser and caches that parsed registry once per process. It
preserves the entire localized seed when no nonempty pool exists for its
language. For an authored pool it retains the shared selector and original
sentence/body behavior. The shared selector's fallback API is unchanged.
The external native tests have not been compiled or executed. Repository policy
keeps Rust builds in CI, and no native acceptance credit is assigned locally.

Focused JavaScript validation runs each file separately:

```sh
timeout 600 node --max-old-space-size=256 --test --test-concurrency=1 rust/tests/web/source-test-placement-twin.test.mjs
timeout 600 node --max-old-space-size=256 --test --test-concurrency=1 rust/tests/web/issue-1569-wasm-worker-inputs.test.mjs
timeout 600 node --max-old-space-size=256 --test --test-concurrency=1 rust/tests/web/worker-unknown-language.test.mjs
```

All 16 cases pass: five placement, five WASM-integrity and six localized-response
checks. All 12 affected policy, core, source-size, naming, worker-budget,
status and release gates pass; Rust formatting, projection verification and
whitespace checks pass. This round changes no shared `data/meta` input and does
not repeat the full gate inventory. Projection verification still reports 49
refused modules, so it does not establish full language parity. Logs are under
`experiments/issue-1580/evidence/checks/opener-native-boundary/`.
An initial gate invocation used the nonexistent name `minimal-core-boundary`;
its failure is retained and earns no verification credit. The actual maintained
gate, `check-minimal-core-boundary`, passes on the corrected invocation.

The original native Spanish routing and full unresolved-reasoning body remain
unverified and unfinished. Inspection finds that the native fallback can promote
a retrieved word sense before checking whether that sense answers the complete
request. That is an investigation hypothesis, not a demonstrated root cause or
a fix. The broader 53 native panic headers, three native budget terminations,
benchmark floors, warnings, recursive context binding, live serving and complete
template/dependency reviews remain open. No ready transition or merge is claimed.
