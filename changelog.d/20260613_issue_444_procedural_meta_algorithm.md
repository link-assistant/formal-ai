---
bump: minor
---

### Added

- **External trusted services are available and opt-out-able (issue #444).**
  The procedural how-to handler may now consult wikiHow, the Stack Exchange
  network, the MediaWiki family (Wikibooks, Wikiversity, Wikivoyage), and
  GitHub READMEs/docs in addition to Wikipedia and Wikidata. Every external
  source is declared in `data/seed/sources-registry.lino` under an
  `external_trusted` group with its own `settings_key` and `default_enabled true`
  (opt-out model), and the web settings UI exposes a section to toggle each one.
- **Procedural how-to / instruction-following benchmark slice (issue #444).**
  `data/benchmarks/procedural-howto-suite.lino` adds self-authored representative
  cases in the style of six widely-used instruction-following benchmarks
  (IFEval, Super-NaturalInstructions, Self-Instruct, OASST1, BIG-bench, MMLU),
  each with a paraphrased held-out variant for anti-memorization, ratcheted by
  `tests/unit/specification/procedural_howto_benchmarks.rs`. Topics span apology
  letters, meal planning, gardening, bicycle repair, pour-over coffee, and
  nutrition labels so the routing is exercised across diverse domains.
- **Central benchmark catalog (issue #444).** `docs/benchmarks.md` indexes every
  benchmark suite the repository has ever touched (issues #103, #304/#317, #362,
  #408, #444) with their fixtures, ratchet tests, sources, and licenses; guarded
  by `tests/unit/docs_requirements.rs`.
- **Grounded meta-algorithm that reproduces topic handlers on demand (issue #444).**
  `data/meta/procedural-howto-recipe.lino` is a machine-readable recipe naming
  every seed role, handler function, evidence stage, Rust↔JS parity target,
  external-service toggle, and benchmark that make up the procedural how-to
  topic, plus eight ordered steps that generalise to any topic.
  `tests/unit/specification/meta_algorithm.rs` keeps the recipe grounded by
  asserting the live source still matches every entry, and
  `docs/meta-algorithm.md` explains how to run and generalise it — so we learn
  from our own source code how to produce changes on the topic rather than only
  emitting one-off code changes.

### Fixed

- **Procedural elaboration follow-ups rebind to the prior how-to (issue #444).**
  After a "how to X" turn, a bare elaboration follow-up such as "Can you give me
  specific instructions?" now rebinds to the established procedure and restates
  the task in both the Rust solver (`src/solver_handler_how.rs`) and the browser
  worker mirror (`src/web/formal_ai_worker.js`), instead of falling through to
  the unknown opener.
