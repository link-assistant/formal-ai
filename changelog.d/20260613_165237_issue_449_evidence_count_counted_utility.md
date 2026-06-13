---
bump: minor
---

### Added
- Symbolic evidence count `C` tracked separately from accumulated utility `U` in
  `src/probability.rs` via `ProbabilityStore::target_evidence_count`, porting the
  interpretable transition model from Kolonin's arXiv:2605.00940 onto the
  associative stack (issue #449).
- Counted-utility decision policy and under-evidenced gating in
  `ProbabilityRankingConfig`: `counted_utility` (rank by `U·C`),
  `min_transition_utility`, and `min_transition_count`. Defaults preserve the
  prior additive behavior.
- `RankedProbabilityCandidate::evidence_count`, surfacing the evidence count next
  to the evidence weight so each ranked option stays locally interpretable.
- Case study `docs/case-studies/issue-449/` with compiled raw data, online
  research, deep analysis, requirement enumeration, and per-requirement plans.

### Changed
- Documented the evidence-count / counted-utility / transition-threshold
  mechanisms in `ARCHITECTURE.md` section 6.1.
