# Requirements delta — 2026-09-15

This refresh queried GitHub issues updated since `2026-09-14T00:00:00Z`, the
current PR #888 body and comments, and its reviews before the final release
proof. It is a delta over the prior complete #1–#1137 audit, not a replacement
for the checked-in historical captures.

## Observed delta

- The paginated issue query returned 12 issue-or-pull-request records and no
  identifier above #1137. There is therefore no unseen issue outside the prior
  #1–#1137 corpus.
- PR #888 has no submitted reviews. Its latest requirement-bearing comment
  says all 32 issue-710 rows are delivered and all twelve focused follow-ups
  (#889–#896, #918, #961, #990, and #991) are closed.
- The comment is corroborated by production-path evidence rather than issue
  state: #991's Rust/server/browser guide synthesis and accessibility-cache
  regressions close R710-20; #990's Electron/POSIX command-stream adapters and
  process regressions close R710-30. The current tally is 31 `works-now`, 1
  `superseded`, and no `still-broken` row.
- Issue #1137 remains open and adds a release-critical requirement: changes to
  `src/agentic_coding/` must exercise Agent, OpenCode, Claude, and Codex before
  merge. Its implementation and regression are included in this PR and traced
  by the R1137 rows.
- The current user acceptance criteria add full first-20 coding measurements,
  dynamic trusted-source rediscovery, release-equivalent local proof, and
  continued branch-Formal-AI authoring. Plans 04 and 05, the R710-D rows, the
  benchmark ledger, and Agent CLI evidence map those requirements to concrete
  production paths and tests.

## Reconciliation rule

Historical before-state evidence stays immutable. Current status documents are
updated only from executable evidence. A closed issue, a local cache, a skipped
workflow step, or a claimed future release is not accepted as completion.
