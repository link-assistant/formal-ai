---
bump: minor
---

### Added

- Constraint-satisfying option networks (`option_network`): research can now record what an answer must supply, what each discovered candidate supplies, and every *minimal* set of candidates that jointly satisfies the requirement — including options made of two separate items, such as a conversion adapter plus the part it adapts. Options are listed cheapest first, with a provenance ladder (authentic, official-compatible, generic-compatible) breaking ties. The network projects onto `world_model::Context`, so the still-open part of a question is an ordinary `ContextDiff`.

### Changed

- Web research now deepens across rounds instead of stopping after one search and fetch. Each round searches only for the aspects of the question no fetched page supports, skips sources already read, and stops when nothing is left open, when a refinement would repeat the previous search, or when the round budget is spent.
- Evidence reading (`option_evidence`): candidates and their prices are now read straight out of fetched page text. The constraints supply the units to look for, so no attribute name is ever matched against prose and the same code reads a Russian spec sheet and an Indian listing. An attribute the page does not state is left open rather than guessed.

### Fixed

- The research loop no longer spends an extra round on questions it has already answered. Deepening now requires a single identifiable gap in a several-aspect question, because token coverage varies with the language a source is written in and a looser rule re-searched answered questions in Hindi and Russian.
