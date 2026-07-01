---
bump: minor
---

### Added

- Grammatical detail for the tomato meaning (issue #538): every surface
  (`tomato`/`tomatoes`, `помидор`/`помидоры`, `томат`/`томаты`) now pins its
  part of speech and grammatical number (singular/plural) in the seed data, and
  the previously missing plural `томаты` was added so both Russian synonyms are
  symmetric.
- New `grammatical_number` semantic facet kind plus `WordForm::grammatical_number()`,
  `WordForm::part_of_speech()`, and `WordForm::denotations()` accessors.
- Grounded, multilingual `grammatical_number` / `singular` / `plural` meanings
  (Wikidata `Q104083` / `Q110786` / `Q146786`) lexicalised in en/ru/hi/zh, with
  cached Wikidata data for offline grounding-closure tests.
- Case study `docs/case-studies/issue-538` with a requirements decomposition,
  per-requirement solution plan, and online research.

### Changed

- Documented the Agent-CLI-driven, meta-algorithm self-hosting development
  workflow as the intended direction in `CONTRIBUTING.md`, `ROADMAP.md`, and
  `REQUIREMENTS.md`, with the aspirational parts of issue #538 recorded as
  tracked follow-ups.
