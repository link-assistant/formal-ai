---
bump: patch
---

### Added
- Grounded three core programming-artifact meanings to verified Wikidata items
  (issue #398, defect #3): `program` → `Q40056` (computer program),
  `code` → `Q128751` (source code), and `sort` → `Q2303697` (sorting, the
  action of arranging objects into order). Each id was confirmed by the
  `scripts/ground-meanings.rs` label-token verifier, with its trimmed source
  snapshot checked in under `data/cache/wikidata/entity/`. The
  `grounded_meaning_coverage_does_not_regress` ratchet floor rises from 136 to
  139 (32.5% of the 428 seed meanings).
