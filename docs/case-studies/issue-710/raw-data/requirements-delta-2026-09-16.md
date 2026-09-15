# Requirements delta — 2026-09-16

The paginated GitHub issue query used `state=all`, `sort=updated` and
`since=2026-09-15T00:00:00Z`. It returned two issue-or-PR records; the only
non-PR issue was [#491](https://github.com/link-assistant/formal-ai/issues/491),
updated at `2026-09-15T15:37:26Z`, with no comments. No new issue number was
found beyond the prior #1–#1137 corpus. This is a delta, not a replacement for
the historical audit or proof that all its requirements are implemented.

## #491: least action includes outcome quality and actual resources

The current issue retains binary decomposition and minimizing elementary work,
while requiring complete input-range behavior when code is simplified. It also
explicitly names elapsed time, computation, memory and user satisfaction as
evaluation dimensions. A shorter path that omits an obligation is not a valid
optimization.

The prior R491-1 audit row already captured binary decomposition and shortest
path/code scoring, with status `unclear`. The new continuation shard makes the
broader acceptance criteria explicit. Current evidence is partial: recursive
split/execution tests, verified-candidate least-action selection, conjunctive
instruction checks and byte-grounded recipe tests. These do not establish a
universal cost optimizer or complete arbitrary-task satisfaction.

## Rechecked external results

- PR #888 remains open and mergeable at remote head
  `2f7a381a28bdaad01016f1a265201a696c9dd869`: 68 successful checks and nine
  intended skips. Local continuation commits have not been pushed.
- The three user-specified test PR heads are unchanged: Kotlin `4ff1d9a`
  (no checks), Scala `a4f344c` (two failed checks), Rust `9d74fb3` (no checks).
  The detailed failures and source-issue requirements remain in Plan 06.

Do not overwrite the earlier audit's captures or equate a closed issue, selected
method, successful file write, or zero-check PR with verified task completion.
