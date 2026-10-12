---
bump: patch
issue: 1154
---

### Fixed

- Codex's `exec_command` sends the shell command under the `cmd` argument
  key; the agentic progress reader recognized only `command`, so a successful
  work-item read was never recorded and the planner re-planned the identical
  `gh issue view` 78 times in one session (6.2M input tokens). One shared
  command-argument reading now serves the whole `agentic_coding` module —
  `command`, `cmd`, `script`, joined array forms, and a schema-driven key
  lookup for clients that name the property differently — and a new
  parsing-independent invariant replaces any planned call that repeats one
  this session already completed with a report of the stuck step and its
  last result.
