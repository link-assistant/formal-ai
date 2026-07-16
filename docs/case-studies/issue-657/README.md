# Issue 657: release self-hosting metric

Issue [#657](https://github.com/link-assistant/formal-ai/issues/657) closes the
measurement gap after E36 self-coding and E37 benchmark-gated promotion. It does
not infer authorship from a Git username: a commit counts only when it links an
exact Formal AI session id to evidence already committed in that snapshot.

## Metric contract

- The release window is `<previous-tag>..<release-input-HEAD>`.
- Merge commits are excluded to avoid double counting their parents.
- Each text commit contributes additions plus deletions from Git `numstat`;
  binary files contribute zero.
- A commit is self-authored only when paired `Formal-AI-Session` and
  `Formal-AI-Evidence` trailers validate. Missing pairs fail closed.
- Percentages are integer basis points, rounded half up and displayed with two
  decimal places.
- The ratchet is the changed-line-weighted share over the latest three release
  rows. A new row may equal or exceed the prior trailing share, never lower it.

The baseline row covers `v0.295.2..v0.296.0^`: 0 of 45,277 changed lines in 16
non-merge commits had qualifying evidence, so `v0.296.0` starts at an honest
**0.00%**. Existing session artifacts are not retroactively claimed because
their commits did not record the attribution trailers.

`scripts/version-and-commit.rs` measures `HEAD` before creating the release
commit, appends the row, then commits and tags it. Both release workflow paths
pass the ledger to `scripts/create-github-release.rs`, which selects the exact
tag row and appends it to the release body.

## Verification and evidence

`tests/unit/specification/self_hosting_metric.rs` builds a minimum Git fixture:
one evidenced Formal AI commit changes three lines and one unmarked commit
changes one, producing exactly 75.00%. It also proves idempotent recording,
release-note rendering, the ratchet failure, and pipeline/ledger wiring.

The in-repo Formal AI agent loop was run before implementation with a literal
file-write-and-verify task; its replay is preserved at
`self-coding-run/session.json`. It proves the local Formal AI tool path was
exercised, but it is intentionally **not** used to attribute this manually
integrated commit. See [requirements.md](requirements.md),
[solution-plans.md](solution-plans.md), and
[raw-data/online-research.md](raw-data/online-research.md).
