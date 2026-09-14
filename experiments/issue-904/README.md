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
* `after-original-fix.txt` — output with the original fix: the `goal` is the
  objective stated after the documented `Issue to solve:` lead, the plan has one
  `Write` step and names no verification command, and the run ends
  "Planned, not executed: …".

`before.txt` was captured by stashing `src`, `data`, and `tests` and re-running
the same example.

## The follow-up changed this shape again

The terminal state above is truthful, and it stayed truthful — but it was also
*every* repository run's outcome, and three production matrices later no
requested artifact had ever been created
([hive-mind#2158](https://github.com/link-assistant/hive-mind/issues/2158)).
A work item names an issue, and an issue URL names no artifact, so recording the
reference was the only end available: the run never read the one document that
says what to build.

The plan therefore now opens with a `Fetch` step for the URL it named, and
`--- tools used ---` reads `["web_fetch", "write_file"]` rather than
`["write_file"]`, because the driver advertises a fetch tool. Once the work item
text arrives, the seed-backed source route and the literal-file composer act on
what the issue actually asks for. `planned_not_executed` survives only where the
capability is genuinely unavailable, the fetch came back empty, or the work item
names no artifact.

`after-original-fix.txt` is kept as the record of the state this experiment was
first captured against. A fresh `after.txt` for the current shape has not been
captured, so re-running the example will not reproduce the file above; the
current behaviour is pinned by `tests/unit/issue_904.rs` instead, which asserts
the read-first order, the executed artifact, and both cases where the honest
terminal state remains.
