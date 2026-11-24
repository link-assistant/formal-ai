# Native reasoning job budget

At source `39196d1fb218d0ef492f90915270f9ae3ac28ba1`, all 27 branch workflows
finish: 24 succeed, one is skipped, Layered CI fails its reviewed CodeQL payload
fingerprint, and CI/CD is cancelled because the reasoning matrix job expires.
The separate pipeline status correctly rejects that cancellation.

The [actual reasoning job](https://github.com/link-assistant/formal-ai/actions/runs/44775337494/job/136423862321)
starts at 16:39:05 UTC and ends at 16:59:06. Its check annotation explicitly says
`The job has exceeded the maximum execution time of 20m0s`. The actual sequential
test result is successful: 148 passed, zero failed or ignored, 972.81 seconds.
Setup and almost three minutes of compilation consume the remaining job clock.
Passing test output does not turn this cancelled job into a successful check.

The complete original stream is retained at
`experiments/issue-1580/evidence/checks/native-reasoning-budget/job-136423862321.stdout.gz`.
It has 1,564 lines: the 148-test start is line 1,334; the successful summary is
line 1,484. Four cases dominate the recorded sequential durations:

| Original case | Observed seconds |
| --- | ---: |
| issue_1157_idempotent_commit / repeated clean commit | 271.174 |
| issue_1160_regular_prompt / restart with stray Kotlin jar | 266.325 |
| issue_1158_pull_request_steps / feedback, title, description and ready | 163.735 |
| issue_1160_regular_prompt / ordinary ready pull request | 137.328 |

These intervals are measured between completed sequential cases, including each
case's subprocess work. They are observations from one hosted run, not estimates
of every future runner's speed or proof that an external tool was independently
authored. No local Rust compilation is performed.

The existing longest-first planner now divides the actual live non-Hive,
non-benchmark listing into `reasoning-1` and `reasoning-2`. Frozen observations
project to 74 cases and approximately 486 seconds per job. Compilation still runs
on each hosted runner. The job timeout remains 20 minutes, matrix parallelism
remains two, each Rust test process has one test thread, and existing complete
`reasoning` and default invocations remain available. New tests receive the
existing 60-second default weight. Empty, duplicate, foreign or insufficient
listings and Cargo failures are rejected before a shard can run unintended tests.

The minimal negative control fails against the original runner with unsupported
`reasoning-1`. The repaired 49-case JavaScript file checks the five-case future
inventory and all 148 actual observed names as exact once-only partitions,
listing reorder stability, failed listing/execution propagation, both measured
weight regenerators and Git's real sparse checkout. The checkout adds only the
one consumed reasoning archive; unrelated historical archives remain excluded.
The expanded native policy file passes 111 cases and the retained production
profile passes 18 cases. Its exact sparse-checkout projection is updated to match
the consumed operand. The initial stale projection failure is preserved.

All three Hive jobs on the old head pass: 20 cases in 325.36 seconds, ten in
795.53 seconds, and seven in 454.09 seconds. Coding, interfaces and benchmarks
also pass. Across the nine native jobs, 528 case executions pass, but the
reasoning job's cancellation still makes that pipeline unsuccessful. This
observation does not qualify the still-unexecuted new split script suites or
the new reasoning matrix. Those require the next published head's hosted CI.

The complete issue remains unfinished. Template comparisons, original-contract
qualification, release and served identity, macro semantic coverage, shared
upstream reports and independent Formal AI delivery retain their existing open
criteria. All supplied reasoning and commands receive zero independent credit.
