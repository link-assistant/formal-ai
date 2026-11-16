---
bump: patch
---

### Fixed
- Hive Mind's formal-ai prompt now binds the delivery to the pull request (#1158): "Update the pull request at <url>." joins "Your prepared Pull Request: <url>" in `prepared-pull-request-patterns`, so the run reads the pull request feedback, sets the title and a description that names the verified artifact, the verification commands and `Resolves <issue>`, and marks the pull request ready. Before this the 2026-09-27 runs left a `[WIP]` draft. The final answer reports each of these steps.
