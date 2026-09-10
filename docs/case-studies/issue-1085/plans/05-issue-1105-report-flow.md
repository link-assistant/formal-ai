# Plan 05 -- the report flow's five live root causes (#1105, closes #1104)

RC1, RC3 resolved earlier; RC8 was #1106 (landed). Remaining, in the issue's
priority order:

- [x] **RC6 -- standalone server export returns a different conversation.**
      Cause (from #1105 §7 and the memory in `formal-ai-report-flow-root-causes`):
      `latest` resolves two ways -- the harness's session id and the server's
      dialog id are different namespaces, and the server picks the newest
      dialog file, not the caller's. Fix: the export endpoint takes the dialog
      id from `x-formal-ai-dialog-id` (see RC5) and, when absent, refuses with
      a machine-readable error instead of guessing. Test: two interleaved
      dialogs, export each, assert each gets its own.
- [x] **RC5 -- harness and server use different identity schemes.** opencode
      sends no `x-formal-ai-dialog-id`; the server hashes message content, so
      two dialogs that start with `Hi` collide (one collided dialog became 52 %
      of the store). Fix: when the header is absent, derive the id from
      (client-declared session if any, else first user message + first
      assistant message + monotonic start time), persisted in the dialog log's
      first line so later turns rejoin it; never from content alone. Test: two
      `Hi` dialogs do not collide.
- [x] **RC4 -- `context learn --session latest` never resolves.** Fix:
      `latest` resolves through the same function the export uses (one
      resolver, `dialog_log::resolve_session_ref`). Test: `learn --session
      latest` after one dialog learns that dialog.
- [x] **RC7 -- success reported while a target failed.** `report_finished`
      checks only the GitHub URL. Fix: it checks every target's result and
      answers with the failed target named; `Final` only when all succeeded.
      Test: gist upload fails (mock) -> answer names the gist, not success.
- [x] **RC2 -- 45 KB pasted inline; gist only above 50,000 bytes.** Fix: the
      threshold becomes a seed value (`report_inline_max_bytes`, default
      20,000) and anything above goes to a gist with a two-line summary
      inline. Test: 25 KB body -> gist + summary.
- [ ] Changelog fragment; #1105 comment with the per-RC verification; PR body
      `Closes #1105`, `Closes #1104`.

Files: `src/dialog_log.rs`, `src/server.rs` (export + report routes),
`src/agentic_coding/report_issue.rs`, `src/solver_handlers/context_learn*.rs`
(confirm names with `grep -rn "session latest\|resolve_session"`).
