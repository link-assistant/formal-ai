# Pull request #1000 — CI/CD diagnostic audit

Pull request: <https://github.com/link-assistant/formal-ai/pull/1000>

Issue: <https://github.com/link-assistant/formal-ai/issues/999>

The technical investigation, requirements, complete timeline, online research,
and test evidence live in the [issue #999 case study](../issue-999/README.md).
This directory records pull-request-specific state and decisions.

## Initial state and discussion audit

PR #1000 was opened as a draft on 2026-08-11 from
`issue-999-31976d2c6ce9` into `main`, with a generated WIP title, placeholder
body, and prepared head SHA `eef51009b3c6edd4924a222806a1cb17210afb76`.

All three GitHub PR discussion surfaces were queried separately and retained
under `raw-data/` and the complete evidence archive:

| Surface | Initial result |
| --- | --- |
| Conversation comments (`issues/1000/comments`) | empty |
| Inline review comments (`pulls/1000/comments`) | empty |
| Reviews (`pulls/1000/reviews`) | empty |

The initial placeholder commit's Coverage, CI/CD, Stock Rust Install, and
External Benchmarks workflows were also downloaded before implementation. They
all completed successfully and correspond to the prepared SHA.

## Decisions made in this pull request

1. Treat the latest cancellation as a capacity boundary only after proving from
   two failed logs and one success that tests remained healthy.
2. Partition the slow platform lane instead of raising its timeout or removing
   portability coverage.
3. Put concurrency at the side-effect boundary: supersede read-only work, never
   cancel a material writer, and retain every pending writer where outputs can
   collide.
4. Preserve true Pipeline Status failures and all unrelated actionlint
   diagnostics rather than making the pipeline cosmetically quiet.
5. Reclassify only results proven informational and repair every file that
   generated a size warning.
6. Reuse established CodeQL, dependency-review, Lychee, and Wayback components,
   but correct the reference template's archived-link false negative.
7. Keep the complete user-required archive under `dev/log/`; provide concise
   issue/PR case studies rather than duplicating multi-megabyte logs.
8. Use a changelog fragment as the patch-release trigger; do not edit the crate
   version manually.
9. Record honest 0% Formal AI self-authorship. The CI/CD and evidence changes
   were manually integrated, so no Formal-AI session/evidence trailers are
   claimed.

## Review checklist

- The final PR description uses `Fixes #999` and replaces the WIP placeholder.
- Every requirement maps to evidence, implementation, and regression coverage.
- The full diff retains existing release/test behavior outside the explicit
  concurrency, sharding, annotation, and gate changes.
- The imported files have immutable-source and Unlicense provenance.
- The branch contains current `main`; the worktree is clean after all commits.
- Local formatting, actionlint, CI/CD tests, data/response tests, Clippy, and
  the full suite pass.
- Fresh GitHub checks correspond to the final head SHA and are green before the
  draft is marked ready.
- No visual UI changed, so screenshots do not apply.
