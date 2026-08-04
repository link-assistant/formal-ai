---
bump: patch
---

### Fixed
- Agent mode no longer reports success for a repository work item whose only steps
  wrote the plan record and read it back: the self-referential verification command
  is gone and such a plan now ends in a `planned_not_executed` terminal state with a
  "Planned, not executed" answer (issue #904).
- A composed plan's `goal` is now the objective stated after the documented request
  lead (`Issue to solve:`, `Task:`, `Goal:`, and their Russian, Hindi and Chinese
  surfaces) instead of the caller's whole system-prompt preamble (issue #904).
