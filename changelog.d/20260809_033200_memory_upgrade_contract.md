---
bump: minor
---

### Added
- Add side-effect-free persisted-memory compatibility preflight and explicit, locked, backed-up, atomic schema migration with JSON receipts and rollback guidance.

### Changed
- Expose memory schema compatibility through `/health` and preserve unknown event metadata across native load/export/write paths.
