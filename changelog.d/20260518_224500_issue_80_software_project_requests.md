---
bump: minor
---

### Added
- Open-ended software artifact requests now route to a `software_project_plan` answer instead of `intent: unknown`, first rendering a Links Notation meaning record with a requirement graph, subtasks, delivery mode, approval gates, reasoning, and plan steps, then returning language-aware starter code after the user approves the plan (issue #80).
