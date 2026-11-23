# Experiment tool review, 23 November 2026

The complete diff for 80 non-test experiment tools in
[PR #1581](https://github.com/link-assistant/formal-ai/pull/1581) was read at
source `bc08222b5a5195dc9992c0459bde2b6369377b9f`, against main
`03a636334975cd7ca5cedff119e2dd0efbc8579f`. All 33 literal chunks were read:
5,763 lines and 342,531 bytes, SHA-256
`bb5645cc736ce9dc3de967884bb4337101106cf18abfe9305edbfd9c142292bf`.

The review covers collectors, template inventories, bounded process probes,
native and release fixtures, governed source refreshes, artifact migration and
routing observations. Local HTTP/compiler/Git fixtures remain distinguished
from hosted or production execution. Historical scripts retain pinned source
identities and explicit limits; this source review does not rerun them or
promote their observations into current behavior evidence.

The command wrapper had two observable defects: an answer-marker substring in
source output truncated the displayed trace, and printed exit-code text could
override the actual command result. The [separate repair](formal-command-output-receipts.md)
adds reproducing tests and actual driver before/after checks. It is a delta
after this pinned review.

The [receipt](../../../experiments/issue-1580/evidence/checks/aggregate-experiment-tools-review-20261123/manifest.json)
retains the whole diff, original inventory, all chunks and completed coverage
ledger. Its original inventory preserves the pre-review false flags; the
separate completed ledger binds the actual reading and exact reconstruction.
Every archive member was read back and verified against byte count and SHA-256.

The experiment test-source inventory has 80 paths, including the older fragment
release-map test outside `experiments/issue-1580`. This corrects the earlier
79-path estimate; its [separate review](aggregate-experiment-tests-review-20261123.md) is now complete. Remaining aggregate
documentation, fixtures and raw evidence review, all applicable template
practices, and the other original issue criteria remain unfinished. Supplied
scripts and conclusions earn zero independent authorship or reasoning credit.
