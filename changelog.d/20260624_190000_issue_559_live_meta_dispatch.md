---
bump: patch
---

### Changed

- Issue #559: replace the solver-local specialized-handler loop with the live
  registry-backed meta method dispatcher. `MethodRegistry` now supplies the
  prelude, specialized, and contextual method ordering used by
  `meta_method_dispatch::try_dispatch`, while the legacy route mapper remains as
  an audit-only parity baseline. Selected handler answers now re-project their
  evidence and Links Notation after the `method` event is recorded, so responses
  expose the selected registry method directly.
