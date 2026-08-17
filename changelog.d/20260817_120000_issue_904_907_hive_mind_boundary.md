---
bump: minor
---

### Fixed

- Read the referenced work item before concluding a repository task cannot be
  executed, so `planned_not_executed` is reserved for a genuinely unavailable
  capability rather than being every repository run's terminal state (#904).
- Route the objective a caller states after an explicit delimiter instead of the
  unmarked harness preamble before it, and stop a caller policy sentence that
  merely mentions a privileged command from selecting it (#907).
