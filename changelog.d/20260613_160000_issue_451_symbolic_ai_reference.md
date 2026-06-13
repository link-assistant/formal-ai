---
bump: patch
---

### Added
- Symbolic AI reference and best-practice audit (issue #451). `README.md`,
  `VISION.md`, and `ARCHITECTURE.md` now cite the Wikipedia
  [*Symbolic artificial intelligence*](https://en.wikipedia.org/wiki/Symbolic_artificial_intelligence)
  article (plus *Semantic network*, *Physical symbol system*, and *Neuro-symbolic
  AI*), making the project's GOFAI lineage explicit: the associative network of
  links is a semantic network in the classical sense.
- A case study under `docs/case-studies/issue-451/` with deep analysis, collected
  issue/PR data, and cited online research (`raw-data/online-research.md`,
  including 2024–2026 neuro-symbolic surveys).
- `docs/case-studies/issue-451/symbolic-ai-best-practices.md` — a 20-row audit
  mapping every technique family the article names to the associative-stack
  component that realizes it (`solver.rs`, `proof_engine/`, `probability.rs`,
  `substitution.rs`, `rule_synthesis.rs`, `knowledge.rs`, `event_log.rs`), with an
  honest applied/partial/proposed status and named reuse targets for each gap.
- Requirements **R298–R304** in `REQUIREMENTS.md` and the regression test
  `tests/unit/docs_requirements.rs::issue_451_symbolic_ai_reference_documents_are_present_and_traceable`,
  which pins the reference, the audit, and the requirement list so they cannot
  silently regress.

This change is documentation and tests only; the solver's behavior is unchanged.
