---
bump: minor
---

### Added

- A deterministic, dependency-free **DPLL satisfiability backend** at
  `src/proof_engine/decision/sat.rs` (issue #451, R305), closing the
  best-practice audit's single proposed gap (§3.2 SAT / constraint solving). The
  solver works over CNF (`CnfFormula`, `Literal`, `SatOutcome`) with unit
  propagation, pure-literal elimination, and chronological backtracking, and is
  byte-reproducible and WebAssembly-safe (lowest-index variable, `false`-before-
  `true` branch order). The Rust crates `splr` / `varisat` were evaluated as
  reuse targets but set aside to keep the engine free of native dependencies;
  they remain the documented upgrade path for CDCL/CSP-scale workloads.

### Changed

- The propositional decision procedure (`src/proof_engine/decision/boolean.rs`)
  now generalizes past the eight-variable truth-table limit: claims with more
  variables are Tseitin-encoded to CNF and handed to the new DPLL backend behind
  the existing "formalize → delegate → trace" seam. An unsatisfiable negation
  yields a tautology proof; a satisfiable one yields a concrete countermodel
  disproof. Claims of eight or fewer variables keep the exhaustive truth-table
  witness unchanged, so every prior proof and test is byte-for-byte unaffected.
- Doubled the propositional-decision test surface: new
  `tests/source/source_tests/proof_engine/decision/{sat,boolean}/tests.rs` unit
  suites plus `tests/unit/proof_request.rs` integration cases exercise the SAT
  path (wide tautology proven via DPLL, wide non-tautology disproven with a
  countermodel, the truth-table boundary, Tseitin-encoding fidelity, and the
  over-width decline), keeping coverage close to 100%.
