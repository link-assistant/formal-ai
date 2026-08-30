---
bump: minor
---

### Fixed

- Answer a Spanish-speaking client's decomposition questions in Spanish.
  `data/seed/multilingual-responses-decomposition.lino` carried English,
  Russian, Hindi and Chinese for all thirteen of its intents and Spanish for
  none, so every reply on the decomposition path — the sub-task list, the
  atomicity verdict, the first step, the depth-bound note, and the two honest
  refusals to enumerate — fell back out of the asked-in language. All thirteen
  now have their Spanish record, pinned per language by an end-to-end test
  (#1066).
