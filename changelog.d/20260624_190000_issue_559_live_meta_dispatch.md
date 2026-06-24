---
bump: patch
---

### Changed

- Issue #559: replace the solver-local specialized-handler loop with the live
  registry-backed meta method dispatcher. `MethodRegistry` now supplies the
  prelude, specialized, and contextual method ordering used by
  `meta_method_dispatch::try_dispatch`, which is now the sole dispatch authority
  (the legacy route mapper and its parity scaffolding were removed outright once
  the registry was proven a behavior-preserving replacement — see R344). Selected
  handler answers now re-project their
  evidence and Links Notation after the `method` event is recorded, so responses
  expose the selected registry method directly.
