# Issue #1455 and the incomplete PR #1456

Base: `3c9195899c627c0e6f05771e1c059bddac2c9ee9` (released v0.355.17).
Working branch: `fix/issue-1455-docs-and-ci-20261029`.

## Failure and scope

PR #1456 fetched the issue through `gh issue view`, then returned
`planned_not_executed`. It did not inspect the repository or update the guide;
its diff contained session logs and a `.gitkeep` change. The failure is missing
execution synthesis, not a successful implementation or an unbounded retry.
A secondary log-upload permission failure does not explain the missing edit.
The minimal public fixture records the original log SHA-256 and source commit;
the raw session log is deliberately not added to this PR.

## Changes

- The associative stack guide documents all three actual integrations and their
  source/manifest, target/feature and host/browser boundaries.
- A comment-preserving JavaScript generator adds explicit docs filters to heavy
  automatic workflows. Lightweight checks, all retained job bodies, schedules
  and manual controls remain. Bot check dispatch also classifies complete file
  pages and rename preimages; unknown data selects normal checks.
- The native detector prevents docs-only automatic release admission even with
  a pending fragment, and treats other Markdown as possible runtime input.
- The JavaScript Formal AI work-item executor reads explicit repository paths
  before an unsupported terminal gap. Its native twin records the same bounded
  observations and decomposition. Failed/partial/policy-refused reads stop.
  JavaScript/TypeScript files are synchronized through `translate-es.mjs`.

The preflight does **not** autonomously author the guide, bind every requirement
or independently verify an implementation. Those stages remain explicitly
unfinished in its result. General evidence-to-edit synthesis is a development
gap; this PR must not receive autonomous task-completion credit for reading files.

## Verification and continuation

Targeted JavaScript files: `docs-only-ci-policy`, `work-item-evidence`,
`fetch-work-item-observations`, `release-pending-pipeline`,
`fast-pull-request-production-profile` and `ci-speed` under `rust/tests/web`.
Requirement assembly/status, package-data mirroring, TypeScript translation and
the deployed staged-release projection are checked with JavaScript tools.
Rust test/compilation belongs exclusively to CI; only the existing source
formatter was used locally. Disk remained about 19 GiB free; no local products,
Rust compilation, Docker builds or broad test suites were run.

Open the combined PR, inspect failures in a batch, and qualify the exact head.
This PR changes runtime/CI source, so it is not itself a docs-only change and may
produce a release after merge. Confirm main and its configured publication chain
before claiming production delivery. Future docs-only changes produce no release
by default; GitHub path-filter diff limits may sacrifice CI savings, and the
detector remains the additional automatic-release guard. Before protecting a
filtered workflow, account for GitHub's pending-check behavior.
