---
bump: minor
---

### Added
- Deformalization reads Lean 4, Rocq and Links Notation, not only first-order logic: the target is matched against the same seeded templates it was rendered from, re-rendered and compared by structure, in both the Rust handler and the browser worker (issue #1186).
- A formalization answer records its parsed clause, each rendered target with its template and the prover lookup as derivation fragments, so `formal-ai explain <answer-id>` shows them.
- A 44-prompt formalization probe set in `data/benchmarks/formalization/{en,ru,hi,zh}.lino`, run by a Rust test and a browser-worker node test; a cross-runtime parity case `e1186_formalization_first_order`; and a no-memorization gate over the formalization seed and runtime.

### Fixed
- Hindi object-verb sentences formalize as a two-place relation through a seeded `verb_final` grammar flag, and the Chinese conjunctive template separates the head from the predicate.
