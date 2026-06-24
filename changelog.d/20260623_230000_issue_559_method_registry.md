---
bump: minor
---

### Added
- Issue #559 (Phase 3): the method registry as first-class link data. The catalogue of handlers each atomic work-unit leaf can route to — the ordered `SPECIALIZED_HANDLERS` table plus the five contextual overrides — is now derived from the live dispatch code (`src/method_registry.rs`, `MethodRegistry::from_dispatch`) and serialized to Links Notation, so the meta algorithm can read and reason about its own methods rather than having them locked away in Rust. The registry is recorded as a trace-only `method_registry` loop event, changing neither routing nor answers, and a grounding test pins every derived method name against `src/solver_dispatch.rs` so the data can never drift from the handlers that actually run. Tracked by REQUIREMENTS.md R331.
