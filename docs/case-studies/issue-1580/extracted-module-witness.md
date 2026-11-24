# Reviewed witness after extracted module repair

The [aaac pipeline](https://github.com/link-assistant/formal-ai/actions/runs/45129261558)
passes, including its producer, release registry checks and all ten native
consumer groups. The complete cohort contains 31 successful workflows, one
Layered CI failure and one explicit skip.

[Layered CI](https://github.com/link-assistant/formal-ai/actions/runs/45129261054)
fails on the governed specification witness after the three explicit Rust child
paths change source bytes. Complete log lines 4521, 4542, 4563, 15343, 20595 and
20616 identify the failing fixture-integrity and Unicode-reader assertions.
The [hosted receipt](../../../experiments/issue-1580/evidence/checks/hosted-merged-head-aaac/manifest.json)
retains the whole logs, paginated jobs, terminal observations and refused rerun
request. No native artifact rerun was issued while the cohort failed.

## Reproduction and repair

Run these existing regression files separately with
`timeout 600 node --test --test-concurrency=1 <file>`:

- `rust/tests/web/rust-specification-fixture-integrity.test.mjs`
- `rust/tests/web/rust-specification-unicode-identifiers.test.mjs`

Both fail with the previous witness and pass after the reviewed refresh:
two and 34 tests respectively. The complete local gate run then passes 89
JavaScript gates, with 54 explicit CI-only skips and zero failures.

Capture the complete candidate with `experiments/issue-1584/review-witness.mjs`,
then review and refresh it with
`experiments/issue-1580/review-extracted-module-witness.mjs PREFIX --refresh`.
The supplied review helper verifies the previous witness, unchanged reader,
current source hashes, six fixture hashes and two source-manifest hashes.
It deep-compares every complete object against that exact eight-field change.
All 223 supported cases, 970 unsupported objects, 1,193 total cases and 141
source records survive. Native behavior is established separately by the hosted
pipeline; refreshing source identities does not establish behavior.

The [repair receipt](../../../experiments/issue-1580/evidence/checks/extracted-module-witness/manifest.json)
binds source commit 6ca491b2905aafafcdfabc2096ac8e0f4d0feb54, before/after serialized objects,
the complete comparison, failing/passing TAP logs and the full local gate run.
The [selected source review](../../../experiments/issue-1580/evidence/checks/selected-production-review-8682/manifest.json)
records inspection of 513 production paths and 56,643 diff lines at the older
8682 source. Exact conservation checks cover mechanical ledger and rename-map
moves. Later source changes and the complete aggregate PR still need review.

All three archives were reopened and each member checked by length and SHA-256.
Supplied review, code and orchestration receive zero independent authorship credit.
Hosted witness qualification, actual artifact reruns, strict style lint debt,
publication authority and remaining original acceptance criteria remain unfinished
in [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581).
