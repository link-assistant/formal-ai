---
bump: minor
---

### Added
- Freely phrased multi-step procedures now reuse the solver's intent formalization and shared source-span decomposition before lowering every ordered requirement into a typed executable operation (issue #674).
- The compiled program carries canonical slugs only, so the English, Russian, Hindi, and Chinese phrasings of the same procedure content-address to one identical set of skill links (issue #674).
- Complete `.lino` procedure artifacts now round-trip through integrity validation and a generic host interpreter; the solver, later "why?" explanation, and Agent planner consume that persisted artifact instead of recompiling prose (issue #674).
- A step outside the vocabulary compiles nothing at all and records a complete review-only learning proposal. Successful seeded paraphrases let the learner infer one typed multilingual candidate without being handed its canonical operation; aliases enter the durable, evidence-bearing ledger only after a green regression suite and explicit human approval (issue #674).
- Formal AI's Agent path writes the same compiled artifact, reads it back, executes it through the public conformance CLI, verifies every step outcome, and returns its source-cited restatement; a reproducible external Agent CLI replay preserves byte-exact artifact and execution evidence (issue #674).
- Every sentence the procedure compiler shows the user — compiled output, later explanation, named gap, and proposal notice — is seeded prose under `compiled_procedure`, `compiled_procedure_explanation`, `skill_gap`, and `skill_gap_name` in en/ru/hi/zh (issue #674).
