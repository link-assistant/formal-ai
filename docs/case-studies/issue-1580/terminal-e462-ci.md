# Terminal e462 hosted CI observations

All 33 workflows for source e46254ac53c360e4c45652fa7abd63fb0d9ecad7
finished: 32 succeeded and one explicitly skipped. Every complete available CLI
log, paginated job response and check response is retained in
[the sealed receipt](../../../experiments/issue-1580/evidence/checks/terminal-e462-cohort-20261123/manifest.json).
Green checks qualify their actual executed paths, not the remaining original issue.

The native producer and ten consumers executed 441 passing group cases,
294 admission cases and 50 registry cases. Ten consumer job intervals actually
overlapped. From producer start to final consumer finish took 613 seconds;
the preceding cohort took 2,757 seconds. These are observed workflow intervals,
not a controlled performance comparison or proof of a single cause.

Layered CI [45240466225](https://github.com/link-assistant/formal-ai/actions/runs/45240466225)
executed all three dialogue owners and all 432 maintained observation cases.
The [complete cost refresh](hosted-shard-cost-refresh.md) retains all 922 file
durations and both warning annotations. Its subsequent scheduling repair needs
qualification on its own hosted source.

## Warning and diagnostic review

The retained analysis classifies 3,981 diagnostic candidates using targeted log
contexts and source inspection. Complete logs are retained; this is not a claim
that every unrelated log line or every aggregate PR diff line was literally read.

| Actual observation | Original complete log location | Status |
| --- | --- | --- |
| JavaScript groups 9 and 7 exceeded 270 seconds | Layered CI lines 3248 and 9514 | Repair drafted; fresh hosted qualification pending |
| OpenSSF scores 2.8 for npm/debug and 2.3 for npm/require-from-string | Security 45240466433 lines 217–218 | Unresolved upstream dependency scores |
| unsafe-libyaml 0.2.11 is unmaintained | Security line 1076 | Existing issue-1017 experimental exception; migration unfinished |
| crates.io publication authority is unknown | Pipeline 45240466910 line 1979 | Token presence does not establish publish authority |

The three `full-replay-installed` failed-phase messages at Layered lines
5270, 5277 and 5283 belong to a passing refusal test at line 5286. Its three
`assert.throws` calls deliberately pin the wrong runtime, source and task;
the verifier emits the failed phase while rejecting those inputs. The workflow
lint's narrow queue-key compatibility exception and malformed-shell canary were
also checked against their maintained source and actual passing job results.

All original failures, warnings and negative controls remain retained.
Broader template practices, dependency migrations, aggregate review and actual
production delivery remain unfinished. PR #1581 remains draft and unmerged.
