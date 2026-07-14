---
bump: minor
---

### Added
- `formal-ai import lexemes` — a deterministic bulk semantics importer that
  generalises the one-off `scripts/ground-meanings.rs` into a reusable pipeline
  (issue #660, R378). It reads a `concepts` document of `<slug> <Qid>` pairs,
  pulls each concept's four project-language labels (en/ru/hi/zh) from the
  committed Wikidata entity cache, and emits grounded meaning blocks whose
  surfaces denote their meaning and carry `part_of_speech`/`grammatical_number`
  facets. `--offline` (the default) reads only the committed cache, so a run
  reproduces the committed batch byte-for-byte; live population is gated behind
  `FORMAL_AI_LIVE_API` and honours the bounded-cache policy `min(1%, 512)`.
- The importer validates on import: every generated block is parsed back through
  the real seed loader and must parse, denote its meaning, and carry both facets;
  a concept that fails is refused and recorded as an `import_rejected` event
  rather than written to the seed.
- A bulk batch of common concrete nouns grounded from Wikidata, growing the seed
  by well over 100 grounded meanings across en/ru/hi/zh, each backed by a
  committed `data/cache/wikidata/entity/<Qid>.{json,lino}` record.
