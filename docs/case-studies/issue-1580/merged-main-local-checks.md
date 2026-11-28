# Merged main and local verification

Main release 0.356.10 at 03a636334975cd7ca5cedff119e2dd0efbc8579f
is merged in e3046ab36b85284eab52e9ed59fd1b9afa2b679c.
The reviewed governed witness is recorded separately in
[the merge receipt](main-03a-governed-witness.md).

The complete 143-entry JavaScript gate inventory ran on that clean, unchanged
source: 88 passed, one failed and 54 require CI. The only failure was
check-status-render. The merged generated table contained two stale copies
of the self-hosting ledger count, 276 and 281; the actual merged ledger has 282
lines. The existing canonical generator replaces those rows with the single
correct row. Neither the generator nor its input ledgers change.

The same status check fails before regeneration and passes afterward. Both
status gates pass after the write. The complete default-branch workflow,
fast-profile and source-warning regression files also pass on the merged
source, together with formatting of the eleven changed Rust files.
No local Rust build runs.

[The lossless receipt](../../../experiments/issue-1580/evidence/checks/merged-main-local-gates/manifest.json)
retains every full-pass stream, the original failure, all focused logs,
the exact before/after page and the generation command. Every archive member
was read back and checked against its recorded size and SHA-256. The focused
repair is not represented as a second clean-source full-inventory result.

Native execution of the latest source, actual artifact reruns, strict style
lint debt, remaining practice acceptance, external publication and the other
original criteria remain unfinished in
[PR #1581](https://github.com/link-assistant/formal-ai/pull/1581).
Supplied work receives zero independent authorship credit.
