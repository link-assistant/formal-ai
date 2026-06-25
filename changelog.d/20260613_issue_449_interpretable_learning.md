---
bump: minor
---

### Added

- Ported the remaining interpretable, non-neural mechanisms from Kolonin's
  "Interpretable Experiential Learning" (arXiv:2605.00940) onto the symbolic
  probability layer (issue #449):
  - `symbolic_cosine_similarity` plus `ProbabilityStore::nearest_similar_evidence`
    implement the paper's `SS` inexact-state fallback — a candidate with no exact
    evidence borrows the nearest stored target's utility, scaled by a
    deterministic bag-of-words cosine, gated by a similarity floor.
  - `ProbabilityStore::reinforce_transition_path` implements the paper's
    episode-wide global feedback — one append-only `markov_transition`
    observation per adjacent state pair, replayable through the event log and
    link-store projection.
  - `ProbabilityDecisionPolicy` groups the `CU`/`TU`/`TC`/`SS` knobs into one
    `Copy` policy, threaded through `SolverConfig::probability_policy` into every
    selection use case via `ProbabilityRankingConfig::with_decision_policy`.
  - `RankedProbabilityCandidate` now exposes a `similarity` field so a
    fallback-driven decision stays locally interpretable.
- Added the `examples/issue_449_interpretable_learning.rs` worked tour of all
  four mechanisms (Bayesian utility, counted utility, thresholds, similarity
  fallback, and episode reinforcement).

### Changed

- Doubled the probabilistic-reasoning specification suite to lock the new
  behaviour and keep coverage close to 100%; existing callers are byte-for-byte
  unaffected because the default policy reproduces the paper's recommended
  baseline (`CU=False`, `TU=0`, `TC=1`, no similarity fallback).
