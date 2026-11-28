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

The first deadline revision reserved 1,620 of 1,800 seconds in one job.
The [native workflow contract](https://github.com/link-assistant/formal-ai/actions/runs/45351919319/job/138300631223)
rejected that 90% budget share at job log line 2123. Keep each deadline and
give the build and capture distinct 30-minute jobs. One locked Cargo
invocation builds both targets in the producer's 720-second phase. The
consumer downloads the exact artifact ID from that producer in the current
workflow run and verifies both command files before use. Its 720-second
capture and two 90-second replays reserve half of the consumer's job clock.

The live capture prints periodic progress when the `FORMAL_AI_CI_VERBOSE`
repository variable is `true`. The child writes its LiNo output
directly. The wrapper's timing summaries and the child's diagnostics reach
both the step log and the retained diagnostic file. Failed phases keep their
nonzero status. Build and capture failures have separate diagnostic artifacts,
and capture failures retain partial output alongside the log. The existing
proposal assertions govern the successful proposals artifact.

Reproduce the failure boundary with
`node --test --test-concurrency=1 rust/tests/web/issue-1571-workflow-diagnostics.test.mjs`.
The deadline replay fails against the original workflow because the live
capture has no own deadline. The headroom replay fails against the first
deadline revision because its phases reserve 90% of the job clock.
The corrected control rejects a missing command and runs the actual step's
shell and the actual budget wrapper against a finite four-second substitute
for the live example. It requires status 124, verifies the retained log, and
checks that partial and successful LiNo output exclude the wrapper's
messages. The proposal-admission suite still verifies read-only behavior.

The input remains the actual live publisher. The timing improvement needed
to complete that live run, and the revised workflow's hosted execution,
remain open. The optional fixture does not earn live execution credit.
