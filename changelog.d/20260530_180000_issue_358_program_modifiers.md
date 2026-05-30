---
bump: minor
---

### Added
- Generalized `write_program` modifiers so operation-vocabulary slugs referenced
  by program-plan rules are discovered as modifiers instead of being hard-coded.
- Added reverse-sorted file-listing program variants, including the composed
  path-argument plus reverse-sort variant across supported template languages.

### Fixed
- Reverse-sort follow-ups to file-listing programs now lower to reverse-sorted
  program output instead of reusing the ascending file-listing variant.
