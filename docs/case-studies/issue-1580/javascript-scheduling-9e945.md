# Retained JavaScript scheduling warning

[Layered CI run 44938456935](https://github.com/link-assistant/formal-ai/actions/runs/44938456935)
on commit `9e945eae3f1f3a03a2b9e3302bdb81000f7820b1` reported budget warnings in groups 2 and 3.
Its 898 complete per-file observations are retained in `experiments/issue-1580/fixtures/hosted-javascript-9e945/`.
The original full log SHA-256 is `32679b18f6b87b086352a6d034d4db9a14c0cf49ec1730ab8925e0b11d07c9a4`.

`hosted-javascript-scheduling.test.mjs` now replays both the earlier and current cohorts.
Before changing weights, the current cohort failed with group 2 modeled at
288.457570 seconds, above the unchanged 270-second warning threshold.
The maintained duration importer raises existing weights from the actual observations;
it retains higher historical costs and the fallback instead of importing faster PR timings as main limits.
Afterward, both cohorts pass and all 906 current suite files remain an exact eight-group partition.
The largest modeled group is 267.049706 seconds.

These four-worker cost models guide scheduling. They are not measurements of the new hosted jobs,
and do not prove a future warning-free wall time. The 360-second budget, 75% warning ratio,
worker count, full test inventory, assertions and failure reporting remain unchanged.
Before/after results, original weights, model output and supplied importer commands are retained under
`experiments/issue-1580/evidence/checks/javascript-scheduling-9e945/`.
Supplied edits and conclusions earn zero independent Formal AI authorship or reasoning credit.

Pull request: https://github.com/link-assistant/formal-ai/pull/1581.

The final maintained entry passes all 323 tests. The file-size, changelog fragment, reconstructed changelog, abbreviation, CI speed and shell lint gates also pass. Raw output and exact digests are retained in the native contract artifact final-checks evidence directory. Hosted compilation, current runtime budgets and the other issue-wide acceptance criteria remain unverified.
