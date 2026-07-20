---
bump: minor
---

### Added
- Freely phrased multi-step procedures now compile into typed, executable skills: `src/skill_procedure.rs` splits a request such as "when I paste a link, fetch its title, translate it to Russian, save both, and reply with the translation" into ordered clauses and maps each onto the step vocabulary seeded in `data/seed/meanings-skill-procedure.lino` (issue #674).
- The compiled program carries canonical slugs only, so the English, Russian, Hindi, and Chinese phrasings of the same procedure content-address to one identical set of skill links (issue #674).
- Compiled procedures stay inspectable: every step records the source sentence span it was read from, and "why did you do that?" re-states the compiled steps with those spans (issue #674).
- A step outside the vocabulary now compiles nothing at all: the solver replies with the named gap ("no compiled capability for …") and records a `skill_gap` event instead of silently dropping the step (issue #674).
