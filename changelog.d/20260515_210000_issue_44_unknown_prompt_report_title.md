---
bump: patch
---

### Fixed
- **Issue #44 — Topbar "Report issue" generates misleading title when session contains unknown-intent responses.** `createIssueTitle` and `createIssueReportBody` now fall back to the last `intent: unknown` assistant message as the effective focus when the user clicks the topbar button (no per-message `focusMessage`). This ensures the generated GitHub issue title reads `Unknown prompt: <prompt>` and the dialog body marks the relevant message as `(reported message)`, matching the behaviour already seen when clicking the per-message "Report missing rule" link.
