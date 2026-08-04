---
bump: minor
---

### Added
- Equation-type corpus with a CI ratchet (issue #891, requirement from #406):
  `data/benchmarks/equation-type-corpus.lino` defines 67 distinct equation
  types — one-step and multi-step linear, `?`/`*` placeholder unknowns,
  symbolic multi-variable isolation, polynomials up to degree five,
  natural-language wrappers in all four supported languages, and
  evaluation/percent flavours — each carrying the exact answer observed from
  the production solver. `issue_891_equation_corpus_solves_every_type` replays
  every case through `FormalAiEngine::answer` and fails below the recorded pass
  count or below 50 distinct verified types.
- Ten recorded `benchmark_limitation` records (irrational and complex roots,
  contradictions, malformed input, identities, unit-carrying equations,
  named-unknown declarations, command-shaped prompts) asserted to keep
  declining loudly rather than fabricating an answer.

### Fixed
- Equation-solving request cues in `data/seed/meanings-calculator.lino`: "solve
  the equation" / "solve equation" (en), "реши/решите уравнение" (ru), "解方程"
  and "求解" (zh), and "समीकरण हल करें/करो", "हल करें/करो" (hi) are now stripped
  before delegation, so `Solve the equation 2 * x + 3 = 11` and its Russian,
  Chinese and Hindi equivalents solve instead of returning a parse error. The
  cues are seed data, so the Rust engine and the JavaScript worker gain them
  from the same source.
