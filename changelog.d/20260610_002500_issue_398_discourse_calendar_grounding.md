---
bump: patch
---

### Added
- Grounded two more meanings to verified Wikidata items (issue #398, defect #3):
  `politeness` → `Q281287` (politeness, the application of good manners) and
  `calendar_today` → `Q3151690` (today, the current day). Each id was confirmed
  by the `scripts/ground-meanings.rs` label-token verifier, with its trimmed
  source snapshot checked in under `data/cache/wikidata/entity/`. The
  `grounded_meaning_coverage_does_not_regress` ratchet floor rises from 139 to
  141 (32.9% of the 428 seed meanings).
