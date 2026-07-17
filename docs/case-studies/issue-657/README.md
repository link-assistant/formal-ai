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

## Formal AI executing issue #657's own task

That replay ran a stand-in task inside formal-ai's own harness — the one harness
that cannot show the capability works over the wire. `agent-cli-learning/`
records the run that does: `experiments/agent_cli_e2e/run_issue_657_metric.sh`
drives two external Agent CLIs (`@link-assistant/agent` and `opencode`) against
this issue's *actual* auto-learning task, with `formal-ai serve` configured as
**their model provider**. Formal AI is therefore executing issue #657's task
using Formal AI, with no external model and no API key.

Both harnesses derived a byte-identical 5,050-byte report over 7 chat rounds.
The parity is the assertion: two tool vocabularies that agree on the artifact
show the derivation is the model's, not a harness's. The step runs in the
`E2E Tests (agent CLI ↔ formal-ai)` job.

The report is derived, never canned. It ranks the persisted attribution network
in [`data/meta/issue-657-self-hosting-learning.lino`](../../../data/meta/issue-657-self-hosting-learning.lino)
through the production `AssociativeMemory` adapter, scoring each observation by
`reads + writes + incoming_links + outgoing_links`. Its four amendments — carry
attribution in commit trailers, start from an honest baseline, weight by changed
lines, ratchet on a trailing window — are the reasoning behind the metric
contract above, recovered from the observations that forced each one. Every
observation is grounded in this issue's measured baseline rather than invented.

The report carries `decision "awaiting_human_review"` and waits on
`promotion_gate "metric_fixture_exact_share_and_honest_ledger_ratchet_pass"`. It
proposes; it does not promote. A learning loop that could adopt its own
amendments to *how authorship is attributed* would be the exact failure issue
#657 rules out — gaming the metric — so the E2E asserts the absence of a
self-promotion alongside the presence of the ranking.
