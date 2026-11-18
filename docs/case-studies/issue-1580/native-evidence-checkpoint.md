# Native failure evidence checkpoint

This is an unfinished checkpoint for [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581).
The issue remains open, the PR remains draft and independent Formal AI reasoning
acceptance remains 0/20. Supplied scripts, commands and patches earn zero
independent authorship credit.

## Published checkout verification

All 23 workflows on `7e89122c9d030b3af5eec2d346141bba3ff3fdab` pass in the
2026-11-18 01:41 UTC snapshot. The pipeline
[44213727766](https://github.com/link-assistant/formal-ai/actions/runs/44213727766)
passes with the reduced source selection. Layered CI
[44213727267](https://github.com/link-assistant/formal-ai/actions/runs/44213727267)
initially fails in checkout before tests: complete log lines 6555–6557 report
curl 28 and an unavailable promisor object, and line 6586 records Git exit 128.
One failed-job retry on the unchanged head passes. The Layered checkout was not
changed by the source-selection patch; this is no transport root-cause proof.
Both attempts, job identities and the successful snapshot are retained under
`experiments/issue-1580/evidence/ci/checkout-7e89122c/`.

## Recover the missing native assertions

The aggregate log from main run
[44188508650](https://github.com/link-assistant/formal-ai/actions/runs/44188508650)
at `015a683197c89e8f74375b097e8dd06a57dcea8f` exposes only 39 distinct native
panic headers. Direct job-log downloads recover all 12 failed native test jobs
and 53 distinct panic headers, including all original 39. Three failed jobs have
no panic header; their budget terminations remain failures. Direct API bytes are
preserved with gzip and SHA-256 identities. The command without
`--allow-escape-sequences` is retained as a failed download attempt.

All 53 headers and the primary payloads of all 14 newly recovered assertions
have been read. This does not claim full literal review of every job-log line.
The download receipt explicitly keeps `fullLiteralReviewComplete: false`.
The corrected inventory, receipts and complete logs are under
`experiments/issue-1580/evidence/ci/main-015a6831/individual-native-failed-jobs/`.

## Candidate changes and original regression contracts

| Change | Cause and preserved check | Verification status |
| --- | --- | --- |
| `559b09a3`: counting membership provenance | A successful retrieved match suppresses the independent seed membership path. Retain seed provenance even when retrieval also matches. The original counting test still requires three instruments, their three seed proofs and exclusion of the spoon. | Formatting and source/projection checks pass; native behavior unverified. |
| `9d14fc9e`: connector observation fixture | A structured completion envelope is followed by a real workspace-source probe. Execute that probe in the fixture's actual workspace before requiring the original Rust program write. | Existing 20 receipt and two parity JavaScript cases pass; this does not execute the native fixture. |
| `782c0241`: agent-mode source trace | The CLI arguments and environment loader moved out of `main.rs`. Trace the serve call through `cli_local_transport.rs` and `solver_helpers/mod.rs`, retaining both activation paths and the other documentation checks. | Formatting passes; native assertion unverified. |
| `9d4832c2`: interpretation fixture input | The first online request and later cache hit produce different evidence for the same interpretation. Set this equality fixture offline. Keep its complete `SymbolicAnswer` equality and ambiguity candidate assertions. | Formatting passes; native assertion unverified. |

The large interpretation payloads are compared field by field without evaluating
the Rust debug output. Intent, answer, confidence, thinking steps and execution
recipe match exactly; evidence and Links Notation differ. The first inspected
differences contain a live capture followed by a cache hit. Full semantic review
of the complete notation diff is not claimed. The raw field identities and diff
remain under `evidence/ci/main-015a6831/`.

The first candidate batch passes eight focused commands: formatting, the 20-case
receipt file, the two-case parity file, source/projection verification, three
size/abbreviation gates and whitespace verification. Complete logs and source
identities are under `evidence/checks/native-evidence-candidates/`. The two later
fixtures pass formatting and whitespace checks, recorded separately under
`evidence/checks/native-fixture-inputs/`. These are not native behavioral passes.
Local Rust builds/execution remain prohibited by the default-branch build policy.

## Catalog and other criteria remain unfinished

Two fresh native Google Trends renderings from this same main head differ from
the committed catalog and from each other: 182,267 and 181,964 bytes versus the
53,208-byte committed artifact. All contain 980 lines. Decoded identities and
complete diffs are retained under `evidence/ci/main-015a6831/`.
The normal engine performs default-on source reads, and offline source clients
still accept warm runtime cache entries. Merely switching off live fetch cannot
establish an immutable catalog input set. The strict artifact/session equality,
all 80 prompt requirements and typed execution-receipt assertions remain intact;
catalog generation and its verified repair are unfinished.

The other recovered assertions, native budget terminations, benchmark floors,
warnings, full eight-template semantic review, dependency migration, recursive
context binding and live serving/delivery requirements remain open. Green PR
scope checks cannot substitute for these acceptance criteria. No ready-for-review
transition or merge is justified by this checkpoint.


Final publication verification passes all five selected JavaScript gates, plus
formatting of all four candidate source/fixture files and whitespace checks.
The complete logs and unchanged source identities are retained under
`experiments/issue-1580/evidence/checks/native-round-publication/`. Fresh 01:57 UTC
feedback is unchanged, main 015a6831 remains merged, all 23 published-head runs
pass and both branch active/queued lists are empty. Native behavior and the
broader acceptance criteria remain unfinished.

The next published head, `b367d603`, has 21 passing workflows and two failures.
The [localized opener and CI metadata checkpoint](localized-opener-and-ci-checkpoint.md)
records both complete primary failure payloads, their candidate corrections,
the reviewed witness refresh and a separate localized opener regression. The
earlier 23/23 result above applies only to `7e89122c`.
