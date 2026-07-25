---
bump: minor
---

### Added
- `formal-ai report body` renders the complete issue-report document — the same
  six sections the web reporter emits — from an exported conversation, with the
  full Links Notation context attached inline or as a gist (#839). One shared
  builder (`src/issue_report.rs`, mirrored for the browser by
  `src/web/app/issue-report.js` and kept honest by a parity test) now formats
  reports for the web, CLI, desktop, Telegram, and VS Code surfaces.
- `formal-ai context session` prints the harness session identifier the current
  shell is inside, so a report can export the session the user is actually in.
- New guide [`docs/report-issue.md`](docs/report-issue.md) documenting the
  report document, the CLI flags, the source semantics, and the gist-visibility
  choice.

### Fixed
- `report issue` now exports the real harness session instead of a hash of the
  conversation's first message. The HTTP server records the
  `x-formal-ai-dialog-id` header of every request, so the exported session is the
  caller's own and two conversations that open with the same sentence no longer
  collide (#839, #838).
- A named `--source` that cannot be exported now fails loudly instead of
  silently degrading to a different capture. Issue #838 was filed with 271 KB of
  base64 HTTP proxy traffic in place of the conversation while the run reported
  success; the server's conversation record is now a separate artifact from the
  proxy trace.
- Oversize contexts are trimmed by whole Links Notation records with an explicit
  `... omitted N records ...` marker instead of `tail -c 12000`, which cut
  mid-record by construction. The complete context is attached as a gist
  (`secret` by default, an explicit documented choice).
- The generated report script verifies every program it calls with `command -v`
  before it runs, and its scratch template expands to a real filename — #838's
  gist was named `formal-ai-report.XXXXXX.lino`.
- Report titles quote what the conversation was about rather than defaulting to
  `Formal AI agentic session report`: the trailing report request is dropped,
  and the first and last remaining user turns are quoted when they fit.
