## Issue #710 Repository Completion and Durable Retention Continuation

The 2026-09-15 maintainer continuation requires complete issue-driven repository
tasks, recursive trusted-source discovery and safe forgetting. Earlier green
benchmark slices do not establish these broader capabilities. Plan 06 in
`docs/case-studies/issue-710/plans/` is the active resumable audit.

| ID | Requirement | Status / evidence |
| --- | --- | --- |
| R710-R1 | Bind implementation language, source destination and output operands independently; preserve ordered output clauses. | Literal stdout composition in Rust, Kotlin, Scala and Python; `process_composition` and `issue_1133_hive_mind_three_runs` test renamed paths and filename-shaped output. Arbitrary program behavior remains open. |
| R710-R2 | Include requested CI, runtime setup, comments, run instructions and executable output assertions. | Source-backed primitive records and shared recipe workflow generation; verifier mutation tests reject wrong case, extra LF and nonzero exit. Live Python project passes. Kotlin local execution still stops at a missing compiler. |
| R710-R3 | Discover missing runtime prerequisites recursively and recover from observed failures. | Open: the live Kotlin failure is retained; generating a setup workflow does not prove local compiler recovery. |
| R710-R4 | A successful tool result must be bound to the requested path, bytes and ordered command before it counts as execution evidence. | `recipe_evidence` and `issue_908` retain actual assistant calls and reject orphan, duplicate, unrelated and out-of-order results. |
| R710-R5 | Commit only the recipe's artifacts, excluding compiler outputs and unrelated staged files. | Explicit-path staging and `git commit --only`; `issue_1133_hive_mind_three_runs` preserves branch/commit evidence assertions. Explicit requests to commit pending repository changes keep their separate semantics. |
| R710-R6 | Preserve original dialogue and observations, unknown legacy records, and imported custom knowledge unless the user authorizes deletion/modification. | Provenance-first classification, apply-time revalidation and duplicate-ID safety; `memory_retention_origin`, `memory_learning`, existing deletion/reset and export/import tests. |
| R710-R7 | Forget only reconstructable cache data; keep the rediscovery recipe and provenance, accounting for retained metadata. | Reconstruction records survive restart and further pressure in `memory_retention_origin`; a public URL authorizes reacquisition, not replacement of historical evidence with today's content. End-to-end automatic source-cache reconstruction remains open. |
| R710-R8 | Formal AI must perform meaningful work through Agent CLI, and failures must become general regression cases. | Live Python project succeeds; Kotlin compiler recovery and the zero-procedure requirement-formalization result are documented as incomplete. Further regression/source authorship remains open. |
| R710-R9 | Retain every requirement as a verifiable obligation; unknown clauses cannot be silently discarded. | Open: existing enumerated-artifact handling and literal-output composition are not a complete obligation graph. |
| R710-R10 | Report proven wrapper/client defects upstream without publishing private traces. | Hive Mind #2259 contains the reviewed MCP transport reproduction; no current Agent CLI defect has been established from the compiler failures. |
