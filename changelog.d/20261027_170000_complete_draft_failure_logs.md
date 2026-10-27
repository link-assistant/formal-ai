---
bump: patch
---

## Fixed

- Recognize Node spec-reporter test evidence in coding research using the same
  strict JavaScript/Rust acceptance rules and shared negative cases.
- Reuse verified same-run Git history for CLI packaging and bounded APT recovery
  for slim runtime and source-builder images, retaining integrity failures.
- Share the pinned official Node/npm payload between full and slim images,
  avoiding the slow distro npm install that left dpkg interrupted at its deadline.
- Copy sparse test fixtures across filesystems and require the new offline CLI
  source import in relocated-stage controls.

- Avoid fetching unrelated release history, keep generated native and TypeScript
  projections current, and use small committed source fixtures in workspace tests.
- Enable Rust panic backtraces and retain large failed-job logs; fix Node 24
  parser loading and test-report compatibility without weakening behavioral checks.

- Retain complete redacted draft console, dialog and native-response evidence,
  explicit process/deadline outcomes, and read-only post-cancellation job logs.
- Refuse evidence-only draft diffs before dispatching the check matrix; useful
  source, configuration and documentation changes still require original
  behavioral acceptance before the task is considered solved.
