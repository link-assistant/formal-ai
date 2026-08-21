---
bump: patch
---

### Changed

- Test-suite environment overrides (`FORMAL_AI_MEMORY_PATH`, `HOME`,
  `FORMAL_AI_DIALOG_LOG_DIR`, the write-path opt-in, and the rest) are now
  scoped to the closure that needs them, through `temp-env`, instead of being
  assigned to the process and put back by hand.

### Fixed

- A test that failed while it held an environment override no longer leaks that
  override into the rest of its binary: the previous value is restored on
  unwind, not only on the success path that ran the restore statements.
