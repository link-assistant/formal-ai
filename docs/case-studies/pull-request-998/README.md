# Pull request #998 — reported-dialog regression repair

Pull request: <https://github.com/link-assistant/formal-ai/pull/998>

Issue: <https://github.com/link-assistant/formal-ai/issues/989>

The technical analysis, requirements, research, and tests-first evidence live
in the [issue #989 case study](../issue-989/README.md). This directory records
PR-specific state and review decisions.

## Initial state and discussion audit

PR #998 was opened as a draft on 2026-08-11 from
`issue-989-66076bee90a1` into `main`, with a generated WIP title, a placeholder
body, and prepared head SHA `3d994fe7218458d6e001ed3cc52ab7190c886b31`.

All three GitHub PR discussion surfaces were queried separately and preserved
under `raw-data/`:

| Surface | Initial result |
| --- | --- |
| Conversation comments (`issues/998/comments`) | empty |
| Inline review comments (`pulls/998/comments`) | empty |
| Reviews (`pulls/998/reviews`) | empty |

`pull-request-initial.json` preserves the initial title, body, branch SHAs,
draft state, and timestamps.

## Decisions made in this pull request

1. Treat the gist as one reported-dialog contract but reproduce every distinct
   failure family with an exact behavioral assertion.
2. Give local dialog and memory control precedence over generic agentic routes
   while retaining all existing agentic behavior for other prompts.
3. Put new natural-language cues and responses in seed data for all supported
   languages, with Rust and browser consumers sharing those entries.
4. Reuse the native link-store projection in Rust and mirror its stable record
   projection in the browser instead of inventing a second memory model.
5. Stop failed search/fetch flows at the stored failure observation; do not
   weaken tool result validation or replace it with string-specific handling.
6. Create three independent secret gists for the three required report links;
   maintain the existing single attachment behavior unless the new report flag
   is explicitly selected.
7. Use a changelog fragment as the patch-release trigger rather than manually
   editing the crate version.
8. Keep self-authorship attribution narrow: only evidence genuinely emitted by
   a live Formal AI / Agent CLI session receives authorship trailers.

## Review checklist

- The PR description closes the exact issue URL.
- Every issue requirement maps to implementation and a regression test.
- Existing formalization, memory, reporting, and web-tool behavior remains
  covered.
- Native and browser behavior agree for the newly supported dialog turns.
- Raw issue/gist/PR evidence and red/green test evidence are retained.
- No visual UI changed, so screenshots and visual regression tests do not
  apply.
- No third-party code or dataset was copied and no dependency was added.
- The branch is merged with current `main`, the worktree is clean, and fresh CI
  corresponds to the final head SHA before the draft is marked ready.
