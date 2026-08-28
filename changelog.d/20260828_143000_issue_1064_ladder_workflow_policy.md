---
bump: patch
---

### Fixed

- Bring the issue #1028 Agent CLI ladder workflow back under the two policies it
  broke when it landed: it now belongs to a concurrency group, so a superseded
  push releases its runner instead of running the ladder twice (#1017), and it
  no longer caches the `target` tree (#534). Both were pinned by tests that
  failed on the commit that introduced the workflow, but the follow-up push
  touched only a shell script, so path filtering skipped the lane that would
  have caught them.
