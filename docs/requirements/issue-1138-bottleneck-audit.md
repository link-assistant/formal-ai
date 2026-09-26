## Issue #1138 Bottleneck Audit

This shard records the requirement contracts introduced while closing the
measured bottlenecks in issue #1138. A status below names the implementation and
the automated test that holds it; it does not claim a run that has not been
observed on the final tree.

| ID | Requirement | Status / evidence |
| --- | --- | --- |
| R1138-B5-1 | Every satisfied obligation node must carry a matching execution record: the command, its exit code or an explicit none, and a SHA-256 of the exact observed bytes. | Implemented by `rust/src/execution_evidence.rs::Evidence::observed`, which hashes its raw byte slice, and by the evidence-bearing `ObligationOutcome::Satisfied` variant in `rust/src/obligation_ledger.rs`. Verified by `rust/tests/unit/specification/execution_evidence.rs` and `rust/tests/unit/specification/obligation_ledger.rs`. |
| R1138-B5-2 | An observation may discharge only the node whose expectation names its command or path; an unrelated result clears nothing. | Implemented by `rust/src/obligation_ledger.rs::ObligationLedger::observe`. Verified by `rust/tests/unit/specification/obligation_ledger.rs::an_unrelated_observation_discharges_nothing`. |
| R1138-B5-3 | A clause with no derivable expectation is split, not discarded; a clause that cannot be split is reported as an unsatisfied gap with its byte span, never as completion prose. | Implemented by `rust/src/obligation_ledger.rs::ObligationNode`, `rust/src/agentic_coding/task_obligations.rs` and the `ObligationStep::ReportGap` branch in `rust/src/agentic_coding/planner.rs`. Verified across ten prompts and five languages by `rust/tests/unit/issue_1138_obligation_evidence.rs`. |
