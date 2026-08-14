## Issue #656 Benchmark-Gated Promotion Protocol

Issue [#656](https://github.com/link-assistant/formal-ai/issues/656) (E37) asks
for a deterministic promotion protocol so self-improvement proposals that pass
benchmark ratchets and CI can be materialized into seed data automatically, while
draft PRs and human review remain the outer gate. It also serves as the open
tracker for R385's fully arbitrary auto-learning, since issue #558 was closed.
PR [#690](https://github.com/link-assistant/formal-ai/pull/690) adds the
`src/promotion.rs` protocol and the `formal-ai improve --promote` command.

| ID | Requirement | Status |
| --- | --- | --- |
| R459 | Define a promotion event protocol in the meta language: proposal link, benchmark evidence links, decision, and applied change, all appended to the event log. | Implemented by `PromotionRun::memory_events` in `src/promotion.rs`, emitting `promotion_proposal`, `promotion_evidence`, `promotion_decision`, `promotion_applied`, and `promotion_rejection` events. |
| R460 | Replay each proposal's benchmark ratchets (coding-modification, industry, unit specs) against the checked-in floors before deciding. | Implemented by `src/promotion/gates.rs`: each canonical command executes once per batch, reports are parsed from process output, and command failure or malformed evidence blocks promotion. |
| R461 | Materialize accepted proposals as `.lino` seed edits on a workspace branch, never a direct push. | Implemented by `src/promotion/materialize.rs`: apply requires a clean Git worktree, creates `promotion/<run-id>`, and writes no remote. |
| R462 | Preserve rejected proposals with their failing evidence, mirroring the R425 `dreaming_candidate_failure` pattern. | Implemented by the `promotion_rejection` event, which keeps the un-applied seed edit and failing benchmark links. |
| R463 | Expose the protocol as `formal-ai improve --promote` (dry-run by default; `--apply` requires `--confirm`). | Implemented by `src/cli_improve.rs`; covered by `tests/integration/issue_656_improve.rs`. |
| R464 | Round-trip promotion events through the bundle export/import path. | Implemented via custom `MemoryEvent` kinds; covered by `promotion_protocol_events_round_trip_through_bundle`. |
| R465 | Document the promotion meta-algorithm and pin it with a traceability test. | Implemented by the promotion section of `docs/meta-algorithm.md` and `tests/unit/docs_requirements_issue_656.rs`. |
| R466 | Treat proposal documents as untrusted intent, not executable benchmark evidence. A proposal must not choose a runner, floor, or observed count. | `parse_promotion_proposals` rejects those fields; the canonical allow-list is derived from checked-in manifests plus the fixed unit command. Covered by `proposal_documents_cannot_inject_runners_or_benchmark_results`. |
| R467 | Enforce every canonical gate policy, including the coding suite's 10,000-basis-point pass-rate requirement, rather than checking only pass-count floors. | `PromotionRatchet::clears` checks command success, floor, and manifest pass rate; covered by `gate_replay_uses_all_canonical_commands_once_and_enforces_pass_rate`. |
| R468 | Execute the learned seed change through Formal AI's Agent task path and verify the authored path and bytes before applying it. | `apply_promotions` calls `run_agentic_task`, extracts its `write_file` arguments, compares them byte-for-byte, and records a content-addressed Agent session id. Exact quote preservation is covered by `general_task_preserves_exact_multiline_lino_payload`. |
| R469 | Promotion must fail closed for unsafe seed paths, dirty/non-Git workspaces, command failures, and malformed benchmark output. | Implemented by `src/promotion/gates.rs` and `src/promotion/materialize.rs`; covered by the failure, malformed-evidence, and non-seed-target tests. |
| R470 | `formal-ai improve --promote` must operate on actual open proposals and must not silently apply a synthetic demonstration proposal. | The CLI requires a non-empty `--proposals` document; demonstration constructors remain test/example fixtures only. |
| R471 | Preserve reproducible real-world evidence, issue/PR feedback, online research, and a requirement-by-requirement solution map. | Implemented by `docs/case-studies/issue-656/`, including external Agent CLI and canonical gate artifacts. |
| R472 | Keep GitHub required checks and human review as the final authority; local replay cannot predict CI on the future branch SHA. | The protocol never pushes or merges. Its branch plan opens a draft PR only after an explicit external push, where required GitHub checks evaluate the actual head SHA. |
