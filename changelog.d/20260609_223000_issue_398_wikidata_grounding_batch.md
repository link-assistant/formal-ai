---
bump: patch
---

### Added
- Added `scripts/ground-meanings.rs`, a re-runnable, self-verifying Wikidata grounding pipeline (issue #398, defect #3). For each curated `(slug, id, expected-label-token)` it fetches `Special:EntityData/<id>.json`, trims it to the cache convention (`type`/`id`/`labels`/`descriptions`/`aliases` in en/ru/hi/zh, wrapped in `{entities:{…}, success:1}`), **verifies** the entity's labels actually contain the expected concept token before grounding — refusing wrong ids such as `Q206` ("Stephen Harper", not "seven") — writes the lossless `.lino` snapshot, and inserts `grounded-in <id>` into the meaning block idempotently.
- Grounded 37 common-vocabulary meanings to verified Wikidata items: calendar weekdays, arithmetic operations, currencies, length/mass/time/data-size units, temperature, mathematical functions, and core quantities. Each id's source snapshot is checked in under `data/cache/wikidata/entity/`, raising grounded-meaning coverage from 18 to 55 `grounded-in` anchors.
- Added the `grounded_meaning_coverage_does_not_regress` data test: a monotonic ratchet that records the grounded-meaning floor (54) so grounding is append-only and progress toward full grounding can only increase.
