---
bump: patch
---

### Fixed

- Store shared GitHub action release metadata once in refreshed dependency snapshots, preserving every sub-action and its immutable release identity while satisfying the existing notation duplication gate. Reject missing, chained, cyclic, conflicting and cross-repository release references.
- Temporarily make duplicate instrumented coverage opt-in while retaining all ordinary critical coding, reporting and release checks; stop artifact consumers after a failed producer and skip unnecessary hosted disk cleanup.
