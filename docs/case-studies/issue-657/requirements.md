# Issue 657 requirements

| ID | Requirement | Verification |
| --- | --- | --- |
| I657-01 | Attribute only commits linked to recorded E36/E37 Formal AI session evidence. | Paired session/evidence trailers are validated against each commit snapshot. |
| I657-02 | Measure changed lines per release window. | Additions and deletions from non-merge, non-binary `numstat` rows. |
| I657-03 | Provide the requested Rust script with `--since <tag>`. | `scripts/self-hosting-metric.rs`; fixture test and direct baseline run. |
| I657-04 | Produce a deterministic percentage from committed data. | Canonical Git range, integer totals, basis-point rounding, exact 75.00% assertion. |
| I657-05 | Cover the behavior with a fixture repository and ledger. | `recorded_formal_ai_evidence_drives_the_release_metric_and_ratchet`. |
| I657-06 | Emit the metric in the GitHub release body. | Release builder appends the tagged row; both workflow paths pass the ledger. |
| I657-07 | Append a `.lino` row for every release. | Versioning records and stages `data/meta/self-hosting-ledger.lino`. |
| I657-08 | Pin ledger and release integration with a specification test. | `release_pipeline_and_ledger_remain_pinned_to_the_metric`. |
| I657-09 | Accept an honest initial 0%. | Committed `v0.296.0` row is 0 basis points. |
| I657-10 | Ratchet monotonically over a trailing window, not a fixed floor. | Three-row changed-line-weighted window; a decrease returns a ratchet error. |
| I657-11 | Make attribution reproducible for contributors. | Trailer protocol documented in `CONTRIBUTING.md`. |
| I657-12 | Preserve issue, PR, feedback, research, and self-coding evidence. | This case-study directory and `raw-data/`. |
