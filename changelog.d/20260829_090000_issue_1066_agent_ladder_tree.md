---
bump: patch
---

### Fixed

- Repaired the #1028 recursive agent ladder so it can run at all: the tree
  generator raised `TypeError` before selecting a single node, leaves claimed
  children they do not have, `run.log` rows were written with literal
  backslash-t instead of tabs, and the per-node instructions reached the agent
  as one line with literal backslash-n in the middle. Covered by
  `tests/unit/issue_1066_agent_ladder.rs` and reproduced end to end by
  `experiments/issue_1066_self_development/reproduce-ladder-tree-generation.sh`
  for #1066.
