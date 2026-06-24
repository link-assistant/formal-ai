---
bump: patch
---

### Added

- Issue #559 (R344): a corpus-wide registry-vs-legacy **dispatch-parity
  certificate** (`src/dispatch_parity.rs`). Where the per-request selection
  comparison (R339) only proves the data-driven method registry never
  contradicts the legacy dispatch authority on the leaves one prompt happens to
  produce, `DispatchParity::audit` enumerates the *entire* route vocabulary the
  system can ever emit — every registered method name (each a self-resolving
  route), every route→method alias (R336), every classifier route slug, and the
  `write_program` intent — and classifies the two authorities on each with the
  same `SelectionAgreement::classify` rule. The single verdict
  `DispatchParity::is_retire_safe` is **zero contradictions**: while it holds, the
  registry is a behavior-preserving drop-in for the hardcoded
  `specialized_handler_name` table — the precondition for retiring it in a later,
  behavior-changing phase. The certificate is derived from live code by
  construction, serializes to Links Notation, and is pure analysis: it changes
  neither routing nor any answer (R13).
