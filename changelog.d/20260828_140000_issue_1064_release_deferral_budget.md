---
bump: patch
---

### Fixed

- Bound how long an automatic release may stay deferred. A policy-ineligible
  cycle still defers rather than turning every push on `main` red, but past
  seven days or twenty pending changelog fragments the same verdict now fails
  the release preflight instead of reporting success. The unbounded deferral
  held 268 commits and 45 fragments behind a green pipeline for 14 days,
  including the fix a downstream consumer was blocked on (#1064).
