# Which route answers a request

```
cargo run --example issue_1069_turn_regression -- '<prompt>'
```

prints the plan the router returns for a one-message conversation. With
`FORMAL_AI_TRACE_REQUESTS=1` it prints the route trace first, which is how the
regression below was located: a turn count says a run went wrong, a trace says
where.

## The regression it found

Issue #1069 moved the evidence-record delivery ahead of the change routes, so
that a leaf asking for an edit *and* for records of it gets all three files.
That also moved it ahead of every registered recipe -- and a recipe is
identified by the artifact it writes, since `LearningReport::matches` looks for
its own `path` in the prompt.

`before-fix.txt` is the run at commit 8f9ea3dc1, on the issue #657 report
prompt. The delivery peels the destination off and re-plans the remainder:

```
[trace] agentic_task=… keep promotion human-review gated, and write self-hosting-learning-report.lino.
[trace] agentic_task=… keep promotion human-review gated,.
[trace] evidence_record=investigating
Some(ToolCalls([PlannedToolCall { tool: "write_file", arguments: "{\"content\":\"repair_case…
```

The residual no longer names the report, so it misses the recipe and reaches
the self-healing route instead: the first thing the run writes is a repair case
nobody asked for, and the report arrives a turn later than it should. Thirteen
`formal_ai_executes_*_through_agent_cli` tests failed on the turn count.

`after-fix.txt` is the same prompt against the fix. Before peeling anything,
the delivery plans the whole request through the routes below it and stands
down when one of them writes the same destination:

```
[trace] evidence_record=declined_settled_route
Some(ToolCalls([PlannedToolCall { tool: "write_file", arguments: "{\"content\":\"self_hosting_learning_report…
```

The check is a probe, not a table of routes that own artifacts: a route's own
state machine decides. Running

```
cargo run --example issue_1069_turn_regression -- \
  'Create file policy/retention.md containing Logs are kept for ninety days; backups are kept for a year'
```

shows why the probe walks several turns -- the literal-file composer's first
step writes `.formal-ai/general-change-plan.lino`, and only its second step
writes `policy/retention.md`. A one-step probe would conclude that nobody owns
the caller's file.
