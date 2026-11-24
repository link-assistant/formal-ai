# File-size consumers and measured JavaScript shard costs

The complete Layered CI stream for published c55a5f9c50ebe58d9bb6103ca46c685e355fcd0d
([run 44988162770](https://github.com/link-assistant/formal-ai/actions/runs/44988162770))
contains 63 failed assertions. Every actual result includes the new empty errors
array; every expected clean result omits it. The seven affected experiment test
files now assert the complete result, including errors. The production collector
still fails incomplete observations. A retained native-test-headroom replay fails
27 cases before the migration; all eleven consumer suites pass afterward.

The same stream warns at 270 seconds in shards two and six. Their final reported
wall times are 306.199 and 331.755 seconds. The first also contains the 63 failed
assertions; completed file-duration events from that failed shard remain valid
cost observations, and are not counted as passing test evidence. All 913 complete
file observations are retained in fixtures/hosted-javascript-c55a5f9c with the
original log digest and a failed-run identity.

Raise-only import increases 55 measured weights. The maintained regression fails
with the published weights and passes with the new weights, preserving eight
shards, four modeled workers per shard, the 270-second warning and 360-second
execution limit. All five scheduling cases and all 26 CI timing cases pass.
This simulation does not establish hosted headroom for the unpublished plan.
The next published round must verify both actual outcomes and timing.

Reproduce with node --test --test-concurrency=1 followed by the individual files
experiments/issue-1580/native-test-headroom.test.mjs,
experiments/issue-1580/hosted-javascript-scheduling.test.mjs and
rust/tests/web/ci-speed.test.mjs. Original preimages and before/after command
streams are retained under evidence/checks/c55-file-size-consumer-repair and
evidence/checks/c55-shard-repair. The complete 142-entry local gate run has 89 passes, 53 explicit skips and
zero failures. Its complete streams and independent weight/hash checks are
retained under evidence/checks/changelog-final-gates. Native execution is not
credited to the skipped gates.

All supplied edits receive zero independent-authorship credit. No local Rust
build or native execution occurs. These repairs do not complete the original
strict budgets, template practice qualification, aggregate review or live
delivery criteria. [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581)
remains draft and unmerged.
