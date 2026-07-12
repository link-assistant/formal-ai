# Issue #540: verified organic learning and runtime regression tests

- Amendments now form for any topic with stated requirements, so organic
  chat-only memory stores (raw messages plus durable task events) learn rules
  even before reproducible specifics exist.
- Refinement folds back only explicit `Learned standing requirement (...)`
  projection marker lines; free-form prose that merely quotes a requirement
  (such as solver fallback text) no longer pollutes rules.
- New regression tests: coverage revocation on rule change, eviction fallback
  for unverifiable records, the full organic record→dream→apply loop through
  the production chat path, refinement resurrection, durable failure records,
  numeric-pattern trial synthesis, multilingual task-kind gating, and the core
  dreaming runtime (idle gate, mid-run yield, `FORMAL_AI_DREAMING` opt-out,
  serve() wiring, locked atomic writes, desktop `PRIORITY_LOW`).
