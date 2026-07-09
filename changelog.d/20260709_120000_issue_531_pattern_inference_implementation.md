---
bump: patch
---

### Added

- Implemented issue #531 pattern inference on a self-contained, link-native
  sequence substrate (`src/sequences/`): a doublet store with structural
  deduplication and lossless expansion, unique symbols, a balanced converter with
  a sequence index and frequency cache, and a Re-Pair-style associative
  compressor with an auditable trace.
- Added 1D sequence and 2D grid pattern inference — repetition, period,
  palindrome, reversal, and translation for sequences; horizontal, vertical, and
  diagonal symmetry, rotations, reflections, and translations for grids — surfaced
  through a `pattern_inference` solver method that analyses concrete "find the
  pattern" / "what comes next" prompts and predicts the next element.
- Seeded a pattern-inference ontology (`sequence`, `pattern`, `repetition`,
  `compression`, `deduplication`, `symmetry`, `rotation`, `reflection`,
  `translation`, `analogy`, `invariant`, `transformation`) rooted in links and
  closed by the total-closure resolver.
- Localized the pattern-inference report into every seeded language (en, ru, hi,
  zh): a response-language follow-up ("answer in Russian") now re-renders the 1D
  sequence and 2D grid analysis — classification, counts, compression, next-element
  prediction, and grid symmetry labels — in the requested language instead of
  stranding it in English.
