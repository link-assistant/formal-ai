---
bump: minor
---

### Added
- Open-ended software artifact requests now route to a `software_project_plan` answer instead of `intent: unknown`, first rendering a Links Notation meaning record with reasoning and plan steps, then returning starter TypeScript code after the user approves the plan (issue #80).
