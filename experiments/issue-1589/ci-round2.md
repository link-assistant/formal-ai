# Second CI investigation

Head 983abf5797edddba4aa095caeee67a90d79b4846 produced 25 completed workflows: 24 succeeded, including Layered CI and Security. Only [CI/CD Pipeline](https://github.com/link-assistant/formal-ai/actions/runs/44172182876), created 2026-11-17T20:42:49Z, failed. Both JavaScript shards succeeded. Native type-checking and linting passed before the focused regression ran.

The full downloaded pipeline log is archived as evidence/ci-round2-44172182876.log.gz. Lines 8044–8046 locate the failure at calendar-currency.rs:30, the evidence assertion on the first prompt. The answer was exactly 45 месяцев with the actual derivation 14 × 1 + 2 × 12 + 7 × 1 = 45 months; 45 / 1 = 45. The worker fixture uses calendar:duration_result:45. Native EventLog::evidence_links projects <kind>:<content-addressed-id>, producing calendar:duration_result:calendar:duration_result_7a10af384eb9018d. The address is not the event payload; this is an existing runtime format difference.

The test now verifies the native event kind/address prefix, unchanged exact shared answer, intent, result, actual arithmetic derivation and rate provenance independently. It collects independent case failures with compact diagnostics, so one failing case no longer hides the rest of the 37-case inventory. Corrected native execution remains required.

Main 058e9bef1214f2f1bc25971243cb787703ed64f1 (PR #1602, issue #1594) is merged, retaining summary vocabulary, tests and projections. The single ledger conflict is resolved from measured sources: 90 files, 36519 total lines, 50 migration files and 18854 migration lines; all dispositions and per-file baselines are retained. The combined tree passes 39 feature/API checks, 14 updated text-transform checks, both projections and Rust formatting. All 138 local gates were rerun: 137 pass or skip, with only the existing unpatched picomatch advisory failing. That dependency limitation is recorded separately from issue #1589 acceptance; the PR Security workflow passed. No local Rust compilation was performed.

Full logs, metadata, annotations, supplied procedures and actual Formal AI receipts are archived. No autonomous authorship is claimed.
