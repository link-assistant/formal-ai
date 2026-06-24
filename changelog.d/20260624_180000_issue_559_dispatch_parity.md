---
bump: patch
---

### Changed

- Issue #559 (R344): completed the **total migration to the data-driven method
  registry as the sole dispatch authority**. An interim corpus-wide parity
  certificate first proved the registry resolved the *entire route vocabulary the
  system can ever emit* — every registered method name (each a self-resolving
  route), every route→method alias (R336, including the `write_program` intent),
  and every classifier route slug — as a behavior-preserving replacement for the
  legacy hardcoded mapper, with **zero contradictions**. With that proof in hand,
  the legacy authority and the parity scaffolding were **removed outright**:
  `src/dispatch_parity.rs` and `intent_formalization::specialized_handler_name`
  are gone, leaving `MethodRegistry::method_for_route` (alias-aware) as the only
  route→method resolver and the only live dispatch path
  (`src/meta_method_dispatch.rs`). The closure invariant the certificate
  guaranteed now lives directly against the live registry — grounded in
  `MethodRegistry::from_dispatch`, `route_method_alias::aliases`, and
  `seed::intent_routing` — pinned by
  `tests/unit/specification/method_registry.rs::the_registry_is_the_sole_authority_that_closes_over_the_route_corpus`:
  no route resolves to an unregistered method, and every method-name and alias
  route resolves.
