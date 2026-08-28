---
bump: patch
---

### Added

- Make task decomposition a recursive binary tree rather than a flat list. The
  contract now carries the `binary` rule — every non-leaf task splits into
  exactly two children — and the Agent-CLI ladder generates the canonical
  63-node depth-five tree at runtime from its 32 atomic leaves, so the structure
  itself is executable and testable. The workflow is manual-only
  (`workflow_dispatch`) with depth and single-node inputs, in its own
  concurrency group so a manual run never disturbs PR CI (#1028).

### Fixed

- Regenerate the self-AST census for `src/task_decomposition/strategy.rs`, which
  the new contract field moved.
