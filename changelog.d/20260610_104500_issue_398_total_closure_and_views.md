---
bump: minor
---

### Added
- **Total reference-closure at zero (issue #398, PR #399 review 4668929105).**
  Widened the closure gate from the `defined-by`/facet/role backbone to *every*
  non-keyword, non-quoted value token in `data/seed/**.lino`. `scripts/close-total.py`
  is an idempotent migration that defines each previously-dangling token as a
  first-class meaning — 17 parent category concepts (intent, task,
  prompt_pattern, source_kind, programming_language, …) rooted at `concept`,
  plus 508 member meanings parented under the category their predicate implies.
  `scripts/audit-total-closure.py` now reports **0** unresolved tokens.
- **Open English WordNet 2024 source.** `scripts/ground-wordnet.py` imports OEWN
  2024 offline (one download, no per-word network calls) and caches 312 English
  lemmas as `.json` + lossless `.lino` under `data/cache/wordnet/en/`, recorded
  under CC BY 4.0.
- **Multi-source `data/view/` merge layer.** `scripts/build-views.py` merges the
  WordNet and Wiktionary lexical caches into 536 per-lemma view entities, each
  with a deterministic `M-<sha1[:12]>` id, a `sources` list, and per-sense
  provenance. Senses sharing part-of-speech with gloss Jaccard ≥ 0.5 merge and
  keep both sources; others stay separate. `--check` verifies no drift, id
  determinism, and merge-threshold correctness.
- **`data/seed/sources-registry.lino`** enumerating every ingested source
  (Wikidata, Wiktionary, WordNet, Wikipedia) with its API endpoint, permissive
  license, and cache path.
- **`tests/unit/total_closure.rs` CI gates** that fail immediately if: any seed
  value token is unresolved (naming offenders); the seed collapses below
  hundreds of meanings; the WordNet cache is absent; `sources-registry.lino`
  omits an ingested source, API, or license; `data/view/` is missing, drifted,
  non-deterministic, or has a provenance-less field; or no view entity is
  genuinely multi-source.
