## Issue #924 Formal AI Self-Development Loop

Issue [#924](https://github.com/link-assistant/formal-ai/issues/924) closes E77
by making one real, reviewable Formal AI contribution a release-cycle
condition. The contract, replay evidence, and requirement map live in
[`../case-studies/issue-924/`](../case-studies/issue-924/).

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R924-1 | Require at least one merged Formal AI-authored pull request in each release cycle. | `ensure_self_development_release` scans first-parent GitHub merge commits and rejects a release range with no matching session-backed PR. |
| R924-2 | Preserve replayable evidence and prove the authored commit reached review unchanged. | The required session/evidence trailers are extended by `Formal-AI-Pull-Request`; the gate matches its number to the merge subject and requires the same commit object in the merged second-parent history. |
| R924-3 | Record each contribution and continue reporting per-release and trailing self-hosting share. | New ledger rows carry repeated `self_authored_pull_request` values plus the existing changed-line measurements; release notes include the reviewed PR URLs and target. |
| R924-4 | Make the release target a non-decreasing ratchet. | Each row records its target. The next target is the greater of the prior target and comparable trailing share; the pre-version release gate rejects a projection below it while the range can still be repaired. |
| R924-5 | Keep E69/E74 dependencies and the ordinary review, CI, and promotion gates intact. | E69 supplies the write-effect floor, E74 supplies the two-way Hive Mind gate, and #924 grants no bypass. The real Agent CLI-authored leaf and raw session evidence are committed in the case study. |
