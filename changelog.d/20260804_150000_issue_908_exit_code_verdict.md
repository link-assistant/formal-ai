---
bump: patch
---

### Fixed
- Step verification in the agentic command reroute now reads the exit code the harness reported instead of guessing from the shape of the output (#908). A verification command that exits `0` without printing anything — `python3 -m py_compile`, `tsc --noEmit`, `diff -q` — is a success, and a command that exits non-zero is a failure even when it printed output. Prose markers decide only when the harness reported no exit code at all, and an `Error: (none)` placeholder field no longer reads as an error.

### Changed
- A failed step is now reported as `Step \`<command>\` for \`<file>\` failed with exit code <n>` in every registered language, instead of the English-only claim that "the agentic CLI harness could not complete" the file — the harness had run the command exactly as asked.
