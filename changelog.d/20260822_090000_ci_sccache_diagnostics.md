---
bump: patch
---

### Fixed

- Start the sccache server explicitly and report its counters between steps.
  `Test (ubuntu-latest / full)` compiled 514 crates while sccache reported one
  compile request, zero misses and zero write errors — a wrapper that is never
  asked cannot miss, so the counter described neither a cold cache nor a broken
  one. The server now starts before any cargo step, and the counters are read
  straight after the step that compiles rather than only post-job.
