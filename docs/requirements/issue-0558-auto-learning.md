## Issue #558 Auto Learning

Issue [#558](https://github.com/link-assistant/formal-ai/issues/558) asks for
dynamic self-programming: Formal AI should learn from failures, represent its
own source as link-native data, regenerate and rebuild accepted changes, reattach
the improved version to the UI, and explain itself from source/data/test
evidence. PR [#637](https://github.com/link-assistant/formal-ai/pull/637)
captures the missing requirements and delivery plan, with special attention to
what PR #601 did and did not deliver for auto-learning, and lands the first
implemented, human-gated slice of the loop in code.

| ID | Requirement | Status |
| --- | --- | --- |
| R387 | Preserve the raw issue #558 source material, related issue #538/PR #601 evidence, GitHub searches, and online research under a dedicated case-study directory. | Implemented by `docs/case-studies/issue-558/README.md`, `docs/case-studies/issue-558/raw-data/`, and `docs/case-studies/issue-558/raw-data/online-research.md`. |
| R388 | Analyze PR #601 and explain why its delivered Agent CLI, diagram, and self-AST slices are not a complete auto-learning system. | Implemented by `docs/case-studies/issue-558/pr-601-gap-analysis.md`, including the stale root requirement status, recipe-driven Agent CLI boundary, partial self-AST, missing Links-to-source, and missing learning-promotion protocol. |
| R389 | Decompose issue #558 into explicit requirements covering failure repair, source-to-links, Links-to-source, recompile/reattach, user-requested self-change, and grounded self-explanation. | Implemented by `docs/case-studies/issue-558/requirements.md`, with requirements `R558-01` through `R558-12`. |
| R390 | Compare existing approaches and libraries that can inform the design without replacing Formal AI's link-native core. | Implemented by `docs/case-studies/issue-558/raw-data/online-research.md`, covering SWE-agent, OpenHands, Reflexion, DSPy, Tree-sitter, rustdoc JSON, syn, and rowan. |
| R391 | Propose a phased solution plan with concrete acceptance gates for safe human-gated auto-learning. | Implemented by `docs/case-studies/issue-558/solution-plan.md`, which defines Phase 0 through Phase 5 and gates for repair cases, source-to-links, Links-to-source, repair execution, learning promotion, rebuild/reattach, and self-explanation. |
| R392 | Protect the issue #558 case-study contract with an automated traceability test and land the work in PR #637. | Implemented by `tests/unit/docs_requirements_issue_558.rs`, wired through `tests/unit/mod.rs`, and tracked by PR [#637](https://github.com/link-assistant/formal-ai/pull/637). |
| R393 | When Formal AI cannot answer an input, compose the failure, the source it maps onto, a benchmark-gated candidate lesson, and a human-review outcome into one auditable, proposal-only repair case (R558-01). | Implemented by `src/self_healing.rs` (`RepairCase`, `RepairOutcome`, `canonical_case`), which reaches a human-gated `AwaitingReview` outcome and never writes source or seed data. Committed as `data/meta/self-healing-case.lino` and covered by `tests/unit/issue_558_self_healing.rs`. |
| R394 | Verify the source-to-links representation round-trips back to source byte-for-byte for a real module (R558-05). | Implemented by `src/self_healing.rs` (`SourceRoundTrip`) over `src/agentic_coding/self_ast.rs`, confirming `source → links → source` reproduces the pinned planner module exactly (`faithful = true`), verified by `tests/unit/issue_558_self_healing.rs`. |
| R395 | Make the self-healing loop reachable through the agentic interface (Codex, OpenCode, Gemini, Agent CLI) and prove it end to end. | Implemented by the fifth recipe `src/agentic_coding/self_heal.rs`, dispatched from `src/agentic_coding/planner.rs`; the driver write and agent-mode server routing are covered by `tests/unit/issue_558_self_healing.rs` and `tests/integration/issue_558_self_healing.rs`. |
