# Validate dependency inventory bytes before effects

NUL-delimited Git output preserves filename boundaries, but decoding it as UTF-8
before validation can silently replace invalid filename bytes. A real Git
fixture with byte FF in an owning directory reproduced two failures: the
JavaScript updater ran four ordinary lock resolutions before its corrupted
working directory failed; Cargo accepted the corrupted complete inventory.

The shared inventory now receives Git's original bytes, checks the terminating
NUL, rejects invalid UTF-8, then decodes the verified observation. Validation
finishes before either resolver can run. Existing failed, truncated, duplicate
and unsafe inventory checks and first-tool failure propagation remain intact.
A path that cannot be represented faithfully is refused explicitly.

Two added regressions create and track the actual byte-valued filename, then
execute each maintained workflow resolver with recording package tools.
Both fail against the unchanged source and pass after the repair with zero
resolver calls. Positive controls also preserve a valid replacement character
and a leading Unicode BOM as literal filename content.

Reproduce with `node --test --test-concurrency=1
experiments/issue-1580/dependency-lock-literal-paths.test.mjs`.
All ten focused, twenty maintained and six neighboring dependency tests pass.
The actual repository inventory still reports three active Cargo manifests,
three preserved Cargo manifests and fifteen npm manifests. Counts overlap;
these observations do not establish all ecosystem coverage or current dependency
acceptance.

The [complete receipt](../../../experiments/issue-1580/evidence/checks/dependency-inventory-bytes-20261123/manifest.json)
retains the original two failures, complete passing logs, commands and original
and repaired sources. Hosted qualification is now recorded below.
This substep does not complete broader updater practice or issue #1580 and
earns zero independent Formal AI authorship credit.

[Pull request #1581](https://github.com/link-assistant/formal-ai/pull/1581)

## Hosted source qualification

[Layered job 137845376042](https://github.com/link-assistant/formal-ai/actions/runs/45218148759/job/137845376042)
executes all ten focused cases successfully, including both malformed-UTF-8
refusal controls. Its tested merge commit has the identical Git tree to branch
head 7ec084a7f; six production/test source identities also match current bytes.
The whole shard reports 963 passes, two skips and zero failures, consuming 202
seconds of its unchanged 360-second budget. These totals overlap the ten cases.

The [hosted receipt](../../../experiments/issue-1580/evidence/checks/inventory-utf8-hosted-7ec0-20261123/manifest.json)
preserves the complete original job, commit/tree observations, all six source
identities and exact passing lines. Literal review covers the listed targeted
ranges, rather than every job-log line. Actual scheduled dependency updates,
broader ecosystem coverage and original issue acceptance remain unfinished.
