## Issue #408 Text And Code Editing Requirements

Issue [#408](https://github.com/link-assistant/formal-ai/issues/408) reported
that a Russian follow-up request to replace text inside the previously generated
Rust code answer fell through to the generic fallback. PR
[#416](https://github.com/link-assistant/formal-ai/pull/416) adds the missing
deterministic edit path, broadens the shared text/code edit operation surface,
and adds the repository-local benchmark profile requested in review.

| ID | Requirement | Status |
| --- | --- | --- |
| R293 | Follow-up replacement requests must edit the active assistant artifact, including generated code, instead of falling through to `unknown`. | Implemented by `src/code_editing.rs`, `src/solver_handlers/text_manipulation.rs`, and the issue #408 regression tests in `tests/unit/specification/text_manipulation.rs`. |
| R294 | Deterministic text/code editing operations must share the multilingual operation vocabulary and keep Rust/browser-worker behavior aligned. | Implemented by `data/seed/operation-vocabulary.lino`, `src/solver_handlers/text_manipulation.rs`, `src/solver_handlers/text_edit_ops.rs`, `src/web/formal_ai_worker.js`, and the shared text-manipulation parity coverage, including replacement, remove, append, prepend, whitespace normalization, case conversion, extraction, counting, punctuation, and line-shape edits. |
| R295 | The issue #408 benchmark-family matrix must list the 8 PR-referenced edit benchmark sources plus 40 additional popular/current LLM benchmark sources, and must provide at least 30 deterministic repository-local prompt-answer variations per source. | Implemented by `data/benchmarks/text-manipulation-suite.lino` and `tests/unit/specification/text_manipulation_benchmarks.rs::issue_408_text_code_edit_profile_passes_local_ratchet`, which requires 48 sources, 30 variations per source, and 1,440 passing checks. |
| R296 | Benchmark documentation for issue #408 must keep the executable local profile, source research, requirement matrix, roadmap, vision, architecture notes, and changelog synchronized. | Implemented by `docs/case-studies/issue-408/README.md`, `docs/case-studies/issue-408/raw-data/online-research.md`, `data/benchmarks/text-manipulation-suite.lino`, and `tests/unit/docs_requirements.rs::issue_408_text_edit_benchmark_scope_documents_are_traceable`. |
| R297 | The issue #408 benchmark claim must be per-source, not aggregate-only: every benchmark source committed to the repository-local profile must pass at least the explicit 10% floor and the stronger 30/30 per-source ratchet, with no issue #408 benchmark work deferred. | Implemented by `text-manipulation-suite.lino` fields `local_10_percent_floor_per_source`, `minimum_pass_count_per_source`, and `minimum_pass_count`, plus `issue_408_text_code_edit_profile_passes_local_ratchet`, which fails unless each source passes 30/30 and the total is 1,440/1,440. |
