---
bump: patch
---

### Added
- Grounded five more conversational/discourse meanings to verified Wikidata
  items (issue #398, defect #3): `greeting_hello` → `Q98815142` (the English
  salutation "hello"), `gratitude_thank_you` → `Q2728730` (gratitude),
  `affirmation_yes` → `Q6452715` (the affirmative particle "yes"), `example`
  → `Q14944328`, and `conjunction_or` → `Q1651704` (logical disjunction). Each
  id was confirmed by the `scripts/ground-meanings.rs` label-token verifier
  before grounding, and its trimmed source snapshot is checked in under
  `data/cache/wikidata/entity/`. The `grounded_meaning_coverage_does_not_regress`
  ratchet floor rises from 131 to 136 (31.8% of the 428 seed meanings).
