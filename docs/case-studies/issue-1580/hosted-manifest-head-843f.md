# Native manifest repair qualification at 843f

The [pipeline run](https://github.com/link-assistant/formal-ai/actions/runs/45114824433)
completed successfully on 843fa303dca9d6b0b1c763b09abcec9d840bd01a.
The terminal cohort contains 33 workflows: 31 succeeded, one failed and one
was skipped. Every one of the ten native contract groups passed.

The native workflow-contract suite passed all 291 tests at whole-log line
66644. Lines 66445 through 66455 record the compact-member, exact rewrite,
malformed-manifest, last-key-wins and escaped-name regression successes.
The original compact package-json regression also passes at line 66457.
This qualifies the structural npm parser and real sparse-checkout repair.

[Layered CI](https://github.com/link-assistant/formal-ai/actions/runs/45114823928)
failed on the same stale checkout assertion in two JavaScript jobs. The
maintained workflow already declared the active harness manifest and recursive
registry children; the exact test expectation omitted them. Commit 07376d152
repairs that expectation. All 168 tests in its complete file pass locally,
alongside the ten real-checkout maintenance regressions.

Whole-log lines 1916 through 1937 report eleven source-size warnings. Ten were
already identified in the preceding run; the additional warning identifies
the 909-line dependency checker. The source-size repair covers these sources
without increasing scanner thresholds or dropping behavior.

Release preflight remains explicit at line 2655: two credentials verified,
zero failed and one unknown. The crates.io publish token remains unknown at
line 2658 because publication authority cannot be established through a
token-accepting read endpoint. Publication is unverified.

The native rerun qualifier stopped on this failed overall cohort before
requesting any rerun. No full rerun, consumer-only rerun or failed-consumer
qualification is claimed for this source.

[Complete terminal observations](../../../experiments/issue-1580/evidence/checks/hosted-manifest-head-843f/manifest.json)
retain the whole pipeline and Layered logs, paginated job metadata, cohort
observations and the rejected rerun qualifier. Every archive member was read
back and checked against its SHA-256. Supplied orchestration receives zero
independent authorship or reasoning credit. The full issue remains incomplete.
