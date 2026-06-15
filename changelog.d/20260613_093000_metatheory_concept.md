---
bump: patch
---

### Added
- Seeded a `metatheory` concept so prompts like `theory of theory`,
  `metatheory`, `теория теории`, and `元理论` resolve to a verified
  `concept_lookup` answer instead of the unknown fallback (issue #436). The
  record carries en/ru/zh summaries grounded in Wikipedia, keeps the existing
  Link Foundation `Links meta-theory` (`теория связей`) routing intact, and is
  grounded for total reference-closure via a new `metatheory` meaning and
  `proof_concept_metatheory` role.
