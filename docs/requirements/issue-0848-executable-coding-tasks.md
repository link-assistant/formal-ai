## Issue #848 Executable Coding Tasks

Issue [#848](https://github.com/link-assistant/formal-ai/issues/848) asks for
an honest, broad measurement of Formal AI doing real coding work through the
Agent CLI. The v0.303.0 baseline passed 38/130 tasks; the prepared branch
passed 45/130 with no test authoring or targeted edit. PR
[#897](https://github.com/link-assistant/formal-ai/pull/897) adds a semantic,
observed execution floor and makes the ladder reject false greens. See
`docs/case-studies/issue-848/` for the full evidence and residual boundary. The
final complete v0.320.0 measurement passes 65/130 tasks (L2 5/12, L3 10/28).

| ID | Requirement | Status |
| --- | --- | --- |
| R848-1 | A complete run must measure all 130 tasks and never confuse a filtered diagnostic or unavailable server with the canonical score. | The runner records completeness and `not_measured`, writes filtered results separately, persists every task incrementally, and requires absence of the launcher success marker before diagnostic text can mean the server never started. |
| R848-2 | Source creation must render executable bytes from the formalized request, never echo request prose, and must observe the exact target after writing. | `src/agentic_coding/code_task.rs` lowers seed-backed function, constant, and test concepts and verifies with an exact `cat`. |
| R848-3 | New Rust targets must compile before the ladder credits them. | The real-client harness invokes `rustc` only for requested targets that did not exist before the task. |
| R848-4 | The full result must have nonzero `test_authoring`, `targeted_edit`, L2, and L3 outcomes. | The complete ladder result records all four above zero; deterministic regressions pin source generation and grounded edits. |
| R848-5 | Repository search must use the named code subject rather than the full conversational prompt. | `src/agentic_coding/shell_command.rs` emits one focused query; seven independent subject shapes are covered. |
| R848-6 | Structured collection edits must transform existing workspace bytes and verify the written result. | `src/agentic_coding/structured_edit.rs` implements read → transform → write → exact observation. |
| R848-7 | The approach must work across supported languages and benchmark facts must track the repository version. | Coding meanings cover en/ru/hi/zh; file-derived expectations resolve the current version from `Cargo.toml` and fail closed. |
| R848-8 | At least 20% of reviewed smallest leaves must be completed through Formal AI and the real Agent CLI. | Sessions `ses_04160c59fffe3FDUKteR56kfQp`, `ses_03d2e0597ffeAUZhq3qAtj2I4U`, `ses_03d2df24effeijLfzPXiUeV4pG`, and `ses_03d2ddb1cffeQkS5gxWjpMojc6` authored four of seven leaves (57%); exact client/server logs, canonical artifacts, and replays are committed. |
| R848-9 | Repeated verified workspace changes may become reusable procedures only after a review-gated learning cycle, and unapproved candidates must remain inert. | `src/workspace_change_learning.rs` separates task and execution fingerprints, records exact successful observations, forms candidates after two distinct tasks, requires a named approval and zero-failure gate, and exposes execution only for content-addressed approved-ledger recipes. |
| R848-10 | Symbol refactors and composite module requests must transform grounded workspace bytes, verify each effect exactly, and terminate. | `src/agentic_coding/workspace_change.rs` uses the shared bounded Normal Markov executor, a compact edit or validated repeated-identifier command, and exact SHA-256 observations; composite requests perform an observed source write followed by a compact registration edit. Issue #848 regressions cover Agent absolute paths, repeated matches, write-only fallback, and both transaction effects. |
