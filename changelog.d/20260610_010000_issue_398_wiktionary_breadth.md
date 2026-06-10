---
bump: patch
---

### Added
- Wiktionary grounding pipeline `scripts/ground-wiktionary.py` (issue #398, open
  item #1 of the `92a29b0` review): it **discovers** candidate lemmas from the
  data — every single-word English surface of a `grounded-in` meaning — fetches
  each from the Wiktionary-backed Free Dictionary API (CC BY-SA 3.0, the same
  source and schema as the existing `en/reference.json`), **verifies** the
  response actually describes the requested lemma, and caches it as pretty
  multi-line JSON plus the lossless `.lino` snapshot via the
  `wikidata_json_to_lino` codec. Idempotent and re-runnable.
- 155 verified Wiktionary entries under `data/cache/wikidata`'s sibling
  `data/cache/wiktionary/en/`, raising the cache from a single placeholder entry
  to 156. Each entry round-trips its full JSON through
  `wiktionary_cache_is_pretty_printed_and_rebuilds_full_json`.
- `wiktionary_cache_breadth_does_not_regress` ratchet (floor 156) so Wiktionary
  coverage is append-only and can only grow as more grounded surfaces are cached.
