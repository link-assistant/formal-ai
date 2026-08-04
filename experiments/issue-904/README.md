# Issue #904 — plan goal held the caller's preamble, verification read the plan back

Reproduce with:

```sh
cargo run --example issue_904_repository_work_item
```

The example feeds the prompt shape from the issue (an agent-harness system-prompt
preamble followed by `Issue to solve: <url>`) through the deterministic general
planner and the agentic replay.

* `before.txt` — output on the parent commit: the plan's `goal` is the whole
  system-prompt preamble, the two steps both operate on
  `.formal-ai/general-change-plan.lino`, `verification_command` is
  `cat .formal-ai/general-change-plan.lino` (the file the run itself wrote), and
  the run ends with "Recorded and verified …".
* `after.txt` — output with the fix: the `goal` is the objective stated after the
  documented `Issue to solve:` lead, the plan has one step and names no
  verification command, and the run ends "Planned, not executed: …".

`before.txt` was captured by stashing `src`, `data`, and `tests` and re-running
the same example.
