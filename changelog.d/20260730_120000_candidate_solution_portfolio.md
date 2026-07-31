---
bump: minor
---

### Added

- Add deterministic parallel candidate-solution portfolios as a domain-independent
  engine: seed-declared draft strategies, a `PortfolioLeaf` trait implemented by
  both arithmetic reachability and rule synthesis, per-draft test ledgers,
  least-action selection, composition backtracking, and multilingual winner
  explanations (issue #704).
- Mine durable `draft_failure` records into per-strategy dreaming-loop lessons,
  so a losing draft becomes retained learning instead of a discarded attempt
  (issue #704).
