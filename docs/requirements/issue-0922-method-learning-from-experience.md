## Issue #922 Method Learning From Experience

Issue [#922](https://github.com/link-assistant/formal-ai/issues/922) closes
E75's first end-to-end learning slice: real recursive-core event logs produce
held-out-validated method proposals, while issue #656's benchmark and human
confirmation boundary remains the only adoption path. See PR
[#1005](https://github.com/link-assistant/formal-ai/pull/1005) and
`docs/case-studies/issue-922/`.

| ID | Requirement | Status |
| --- | --- | --- |
| R922-1 | Propose at least one reusable method abstraction from real solved-problem event logs and validate it on unseen experience. | Delivered: `method_learning` executes the production recipe for two support problems and one held-out problem, reusing `algorithm_discovery` to infer and validate recurring event-kind sequences. Pinned by `rust/tests/unit/issue_922_method_learning.rs`::`real_event_logs_propose_an_inert_method_that_is_adopted_only_after_promotion`. |
| R922-2 | Keep all learned candidates inert until benchmark-gated, human-confirmed promotion. | Delivered: Discovery returns proposal data only; it cannot supply gate commands or observations, and production loads only the checked-in promoted seed. Pinned by `rust/tests/unit/issue_922_method_learning.rs`::`real_event_logs_propose_an_inert_method_that_is_adopted_only_after_promotion`. |
| R922-3 | Adopt at least one validated method into the live registry through the promotion path. | Delivered: Run `promotion_run_21bc44690947f221` cleared the 4/4 coding, 13/13 industry (curated; beside the latest upstream rows HumanEval 14/164 on the full slice (`--online`, 2026-09-17) and MBPP 49/500 cold-offline (2026-09-18), rendered per suite in `docs/benchmarks.md`), and 12/12 unit floors before `--apply --confirm` materialized `learned_recursive_core_740155f4b5796f69`; `MethodRegistry::from_dispatch` now exposes that learned record. Pinned by `rust/tests/unit/issue_922_method_learning.rs` (the production registry must contain the promoted record) and `rust/tests/unit/docs_requirements/method_learning_from_experience.rs::issue_922_promotion_and_agent_cli_evidence_are_reproducible`. |
| R922-4 | Preserve rejected candidates and exact reasons. | Delivered: The discovery run retains held-out mismatch details, while a blocked benchmark produces an append-only `promotion_rejection` carrying its suite-specific reason and evidence link. Pinned by `rust/tests/unit/issue_922_method_learning.rs`::`rejected_benchmark_proposals_are_durable_and_include_the_reason`. |
| R922-5 | Preserve recursive recipe/source parity and the existing regression floor after adoption. | Learned records are separate from compiled handlers, so dispatch order is unchanged; focused parity plus the complete contributor check suite guard the floor. Pinned by `rust/tests/unit/issue_922_method_learning.rs`, which requires the adopted method to expose no compiled route. |
| R922-6 | Preserve reproducible research, case-study, release, and real Agent CLI evidence. | Delivered: primary-source research, raw GitHub snapshots, reviewed proposal input, promotion outputs, independent exact-byte Agent CLI replay, traceability tests, and the minor changelog fragment live in the issue case study and PR #1005. |
