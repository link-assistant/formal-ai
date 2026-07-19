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
  a concept that fails is refused, persisted as a replayable `import_rejected`
  event, and leaves the previous shard set unchanged. Each accepted surface
  carries its exact Q-record JSON field, language scripts are checked with a
  deterministic same-record alias fallback, obsolete importer-owned shards are
  removed, and the CLI reports requested/accepted and expected/emitted coverage.
- A bulk batch of common concrete nouns grounded from Wikidata, growing the seed
  by 208 grounded meanings and 832 surfaces across en/ru/hi/zh, each backed by a
  committed `data/cache/wikidata/entity/<Qid>.{json,lino}` record.
- A generalized, human-review-gated learning report derives reusable importer
  amendments from persisted observations. CI executes the task through two real
  Agent CLI clients with Formal AI as their model provider and requires the
  resulting reports to be byte-identical.
