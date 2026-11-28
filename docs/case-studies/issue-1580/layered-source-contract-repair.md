# Layered source contract regression repair

[Layered CI run 45039178952](https://github.com/link-assistant/formal-ai/actions/runs/45039178952)
tests source `b8383d52b10fb8be79f8fe7e42944b88343a07dc`. Its complete retained
log reports a stale inventory-call assertion at lines 4111–4128 and a stale
eight-shard assertion at lines 7318–7350. The documentation receipt failure at
lines 7353–7383 is the same already repaired fixture failure described in
`dispatch-pagination-and-context.md`.

The inventory helper now receives `changelog.d` as a checked argument and returns
read errors before release decisions. Keep the regression's directory and error
propagation assertions and match the actual call through the release eligibility
function. The compiled CLI's existing unreadable-inventory cases retain the
behavioral check; local source matching alone does not execute Rust.

The complete-suite test now reads main's shard matrix, requires a contiguous
nontrivial range and checks equal matrices and counts for PR, feature dispatch
and main. It executes every planner shard in all three contexts, still requiring
every default web suite exactly once. This preserves coverage when the measured
budget calls for a different shard count.

Both test files fail before these fixture corrections and pass afterward: three
release-pending tests and two full-default-profile tests. The first local repair
also exposed the changed release-call shape; its failed log remains retained.
Run each file separately with `node --test --test-concurrency=1`.

[Lossless records](../../../experiments/issue-1580/evidence/checks/layered-source-contract-repair/manifest.json)
include the complete hosted log, job identities, both before-controls and local
checks. Hosted verification and the wider issue remain unfinished. Supplied
patches and controllers receive zero independent authorship credit.
