# Aggregate fixture review, 2026-11-23

This source review covers 24 changed Rust fixture paths and the picomatch
compatibility README between main 03a636334975cd7ca5cedff119e2dd0efbc8579f
and bc08222b5a5195dc9992c0459bde2b6369377b9f in
[PR #1581](https://github.com/link-assistant/formal-ai/pull/1581).

All 20 literal diff chunks were read: 3,532 lines, 148,692 bytes, SHA-256
8030912ebf2c48374e5f99d366da20bd728a8f89fdd5625efa148a5a88c86a03.
Independent reconstruction verifies each chunk, the whole Git diff and all
25 working files against the bound source. The original unread inventory
remains alongside the completed review ledger.

The changes retain the prior 15 executor cases and all 22 provenance cases
without semantic changes; a strict duplicate-key JSON reader independently
checks those conservation claims. Ten executor cases are added. Calendar,
quote-boundary, unverified-status and independent-synthesis fixtures retain
negative expectations. Generated Rust translation fixtures explicitly carry
unsupported constructs rather than claiming complete translation parity.
The offline publisher and delivery fixtures remain mocked evidence.

The binary CodeQL fixture was decompressed with a 1 MiB bound. Its 75,971
bytes have SHA-256 e4fcbe037ed79ac6c83338fc52e5888e4d2906852bb2ca319188db768bd1aa1e.
All 15 package projections were read, including declared dependencies,
feature maps, resolved dependencies and compiled artifacts. An independent
comparison checks every selected field against the original snapshot:
15 metadata packages and resolve nodes, 14 compiler artifacts, and no
compiled artifact for serde_core. Descriptive Cargo package metadata was
not reviewed literally. The entire compressed original is retained.

The [sealed receipt](../../../experiments/issue-1580/evidence/checks/aggregate-fixtures-review-20261123/manifest.json)
contains the complete diff, 20 chunks, original and completed inventories,
binary source, projections, conservation script and result. All archive
members were read back and checked by byte count and hash. Its archive has
30 members, 55932 bytes and SHA-256 a77c42ecb26d3f06ae758d4d88d498d32da6aaf61491e533b992a2342df8dc49.

No new Rust build, test execution, live delivery, CodeQL semantic analysis
or independent authorship is credited. This source subset is reviewed;
the full aggregate review and the wider issue acceptance criteria remain
unfinished. Overlapping historical inventories are not unique totals.
