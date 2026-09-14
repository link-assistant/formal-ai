# Issue 919 requirements

| ID | Requirement | Implementation and verification |
| --- | --- | --- |
| R919-1 | Start with a coding task that exhausts the existing synthesis routes and records a stable skill gap. | The end-to-end regression invokes the real Ruby `count_to_three` request, asserts `write_program_skill_gap`, and carries its `program_skill_gap` identity into `CodingResearchGap`. |
| R919-2 | Plan research, fetch exact external material with provenance/cache, and formalize it into the meta-language. | `research_coding_skill_gap` derives a deterministic task/language query, uses `execute_source_research`, requires the versioned licensed source shape, and emits a content-addressed `coding_procedure` Links Notation record. |
| R919-3 | Compile a candidate with existing #897 machinery and keep it only if bounded workspace execution proves it. | The candidate is a `KnowledgeKind::Procedure` and calls `execute_workspace_rewrite`; exact expected output is an immutable gate. Failure rejects the cycle version and leaves the durable ledger empty. |
| R919-4 | Mark researched procedures and preserve full source, license, fetch, and verification provenance. | Every ledger procedure has `origin research`, `status execution_verified`, query, URL, SPDX expression, fetch time, source SHA-256, formalization id, executor, verified-output SHA-256, step count, and reviewer. Ledger and procedure ids are content-addressed and checked on restore/use. |
| R919-5 | Use the same verification path as hand-seeded procedures and respect the completed E69 dependency. | Both learning and held-out replay call the #897 `verified_workspace_rewrite` executor. E69 was completed by issue #916 / PR #966 before this branch. |
| R919-6 | Make live retrieval opt-in and CI replay deterministic from cache. | `CachedSourceClient` defaults offline; the test explicitly enables its first client, then reconstructs a default-offline client and proves zero additional transport calls plus byte-identical proposal/ledger/output. |
| R919-7 | Failed research must update the gap and schedule another round. | `CodingResearchGap` appends query, failure reason, and status, then plans `alternative evidence round N`; the wrong-output regression asserts no procedure was retained. |
| R919-8 | Preserve implementation reasoning and external research in the parent case study. | This directory records root cause, component reuse, requirement mapping, primary standards, scope, and verification. |

## Alternatives rejected

- Saving a search answer as a procedure would collapse evidence and executable
  authority. The implementation requires a typed operation and execution gate.
- Executing arbitrary code copied from a page would bypass the bounded #897
  executor. The v1 source shape permits only `verified_workspace_rewrite`.
- Storing only a URL would make later review and replay depend on mutable remote
  state. The ledger binds URL, fetch time, and exact bytes by SHA-256.
- Retrying the same query after failure would not learn from the gap. The gap
  record changes the next deterministic query while preserving earlier rounds.
