---
bump: patch
---

## Fixed

- Retain complete redacted draft console, dialog and native-response evidence,
  explicit process/deadline outcomes, and read-only post-cancellation job logs.
- Refuse evidence-only draft diffs before dispatching the check matrix; useful
  source, configuration and documentation changes still require original
  behavioral acceptance before the task is considered solved.
