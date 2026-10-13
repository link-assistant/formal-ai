---
bump: patch
---

Publish prepared Docker images directly to the registry instead of exporting a full local image before pushing. Stream plain BuildKit progress and bounded operation timings while retaining source, executable, digest, runtime and anonymous-access verification. Existing cache settings and execution budgets remain unchanged.

The previously published v0.353.1 tag and crate remain unchanged; its Docker and GitHub release targets were blocked by a publication timeout. This patch selects a new release version, normally 0.353.2. Zero Formal AI authorship is credited for supplied orchestration.
