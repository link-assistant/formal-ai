---
bump: minor
---

### Changed
- Spider-Man release-order answers are no longer a frozen sentence in
  `data/seed/facts.lino`. They are rendered at question time from a
  source-backed snapshot: a checked-in SPARQL query against the Wikidata Query
  Service (`data/cache/wikidata/query/spider-man-title-role-films.rq`), its raw
  answer, and the cached Wikidata labels of every film (issue #892).

### Added
- `data/seed/release-timelines.lino`: a general release-timeline registry —
  per-language answer wording plus, for each timeline, its source, query, cache
  file, snapshot date, freshness window, SHA-256, and the dated works with their
  localized titles. `scripts/ground-release-timelines.py` regenerates it from
  the cache (`--check` verifies, `--refresh` re-fetches).
- `formal_ai::release_timeline`: renders a timeline for a language as of a given
  day, ordering released works by date, listing announced ones separately, and
  switching to stale wording once the snapshot outlives its freshness window.
- `release_timeline:*` evidence links recording which snapshot an answer was
  computed from, its digest, the released/announced counts, and staleness.
