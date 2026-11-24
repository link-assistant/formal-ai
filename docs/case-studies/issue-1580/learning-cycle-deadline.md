# Scheduled learning cycle deadlines

The original [15 November run](https://github.com/link-assistant/formal-ai/actions/runs/43706083454/job/132787346040)
exited with status 101 at log line 516. Its shell redirected Cargo stderr to
a file and stopped before printing or uploading it. The later stderr trap
exposed the failing phase. A [22 November run](https://github.com/link-assistant/formal-ai/actions/runs/45017007629/job/137191364713)
successfully replayed the recorded frontier and uploaded its proposals.

The [23 November run](https://github.com/link-assistant/formal-ai/actions/runs/45175845066/job/137696745267)
on `03a636334975cd7ca5cedff119e2dd0efbc8579f` took a different path. It used
the live Google Trends frontier. The whole job reached its 30-minute limit.
The captured stream records a successful 5 minute 49 second build at line
1389 and the example invocation at line 1390. GitHub canceled the step at
line 1391 and terminated the example and a curl subprocess at lines
1408–1409. Both the assertions and the proposal upload skipped. The report
evaluates each generated prompt with normal solver routing and can incur
external reads. The retained log shows the phase that stalled. It does not
identify the prompt, route or remote service that consumed the time.

The workflow builds both targets in a single locked Cargo invocation. The
build and live capture each have a 720-second execution budget, and the two
replays each have 90 seconds. Their sum leaves 180 seconds of the existing
job limit for runner setup, termination grace and uploads. The live capture
prints periodic progress. The child writes its LiNo output directly, while
the wrapper's timing summaries and the child's diagnostics reach both the
step log and the retained diagnostic file. A failed phase keeps its
nonzero status and uploads the diagnostics separately. Only the existing
assertions can admit the proposals artifact.

Reproduce the failure boundary with
`node --test --test-concurrency=1 rust/tests/web/issue-1571-workflow-diagnostics.test.mjs`.
The new regression fails against the previous workflow because the live
capture has no own deadline. The corrected control runs the actual step's
shell and the actual budget wrapper against a finite four-second substitute
for the live example. It requires status 124, verifies the retained log, and
checks that partial and successful LiNo output exclude the wrapper's
messages. The proposal-admission suite still verifies read-only behavior.

The input remains the actual live publisher. The timing improvement needed
to complete that live run, and the revised workflow's hosted execution,
remain open. The optional fixture does not earn live execution credit.
