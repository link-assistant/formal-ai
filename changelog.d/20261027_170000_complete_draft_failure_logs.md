---
bump: patch
---

## Fixed

- Avoid fetching unrelated release history, keep generated native and TypeScript
  projections current, and use small committed source fixtures in workspace tests.
- Enable Rust panic backtraces and retain large failed-job logs; fix Node 24
  parser loading and test-report compatibility without weakening behavioral checks.

- Retain complete redacted draft console, dialog and native-response evidence,
  explicit process/deadline outcomes, and read-only post-cancellation job logs.
- Refuse evidence-only draft diffs before dispatching the check matrix; useful
  source, configuration and documentation changes still require original
  behavioral acceptance before the task is considered solved.
