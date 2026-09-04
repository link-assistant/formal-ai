# Issue 924: Formal AI Self-Development Loop

Issue [#924](https://github.com/link-assistant/formal-ai/issues/924) closes E77
by turning the existing self-hosting measurement into a recurring release
condition. This work depends on E69's merged write-effect foundation and E74's
full-circle Hive Mind integration in
[PR #1004](https://github.com/link-assistant/formal-ai/pull/1004). Direct Agent
CLI execution remains the supported route when Hive Mind is not the driver.

## Root cause

Issue #657 could attribute session-backed commits, publish their changed-line
share, and report a trailing-window regression. It did not require a real
repository contribution in each release. A release could therefore keep
recording zero authored commits indefinitely. The ledger also had no durable
PR identity, and the release-time metric could not distinguish a reviewed
change from a direct commit carrying an unverified claim.

The earlier trailing-share ratchet could only warn during release recording.
That was necessary for historical malformed ranges, which are immutable once
on `main`, but it left no actionable release boundary at which a missing
contribution or a falling target had to be repaired.

## Release-cycle contract

`Formal-AI-Pull-Request` extends the existing session and evidence trailers.
The release gate walks first-parent GitHub merge commits in the tag-to-HEAD
range and accepts a contribution only when all of these facts hold:

1. at least one non-merge commit introduced by the pull request has valid,
   committed Formal AI session evidence;
2. every *attributed* commit it introduced names that canonical GitHub
   pull-request URL;
3. the matching `Merge pull request #N` commit contains the same commit object
   on its second-parent branch and not on its first parent; and
4. at least one qualifying PR exists in the release range.

A fabricated trailer on a direct commit therefore cannot satisfy the floor, and
a trailer naming some other pull request disqualifies the one that introduced
it. The ledger writes every qualifying PR URL into the release row beside the
per-release and trailing changed-line shares.

Condition 1 originally read *every* non-merge commit, and condition 2 applied to
every introduced commit rather than to the attributed ones. Issue #1069 narrowed
both: the old pair measured the composition of a pull request rather than the
authorship of the work, so a self-authored change could never ride along beside
a human commit and every contribution needed a pull request of its own. The
measured share is unaffected, because it is computed per commit -- an
unattributed commit stays in the denominator and out of the numerator either
way.

The target for a release is the greater of the prior target and the prior
comparable trailing share. The candidate release's projected trailing share
must meet that target. Because this check runs before version files or the
release commit are written, failure is actionable: leave the range open, merge
additional reviewed self-authored work, and retry. An existing row for the same
tag is excluded from its own projection so release retries stay idempotent.

## Unchanged gates

The new release condition grants no write, approval, CI, merge, or promotion
authority. Formal AI's commit must land byte-for-byte through the same normal
pull-request workflow as every other contribution. Repository review and CI
judge the branch; applicable seed/runtime promotion still uses the existing
trusted-gate protocol from #656. The self-development gate only verifies the
already-merged ancestry and release target. It cannot waive or manufacture a
review, check result, or promotion decision.

## Replayable self-authorship

The smallest real leaf in this change was produced by Formal AI through the
actual Agent CLI. Session `ses_0020cec63ffe7RIFkQ1qH9YZcY` authored
`self-hosting-authorship/release-invariant.txt`; the exact task, raw Agent
stream, Formal AI server trace, diagnostic classification, session id, and
workspace status are preserved beside it. Commit `58d60769` carries all three
trailers, including
[PR #1007](https://github.com/link-assistant/formal-ai/pull/1007).

Regenerate the evidence after building the debug binary and installing the
Agent CLI:

```bash
experiments/issue_924_agent_cli.sh
```

The runner boots the candidate Formal AI server, invokes the real Agent CLI,
and byte-compares the requested artifact. The committed raw evidence is the
replay record; generated transcripts do not move the changed-line numerator or
denominator.

## Incremental self-development execution

The maintainer follow-up asked for more than one isolated Agent-authored file:
run the same task through Formal AI, use auto-learning, and recursively split
only what a real attempt proved unsolved. The replay in
`incremental-self-authorship/` does exactly that through the production
`formal-ai agent dispatch --incremental --cli agent` path.

The compound task first asked for a coordination effect and two canonical
self-development contracts. Its exact verifier rejected that whole-task
attempt, so the shipped task splitter produced three independently verified
children. Four native Agent sessions are retained: the failed whole attempt and
the three child attempts. The child effects compose into the real workspace;
the two Links Notation contracts are copied byte-for-byte into `data/meta/`.

That real run uncovered a general orchestration defect. After every child had
passed, the controller always invoked the parent agent again. The redundant
call rewrote a correct composed effect and made the root fail. Incremental
dispatch now supplies the parent task to its allowlisted verification command
and records a non-mutating `composed-verifier` replay when the composed
workspace already passes. If there is no verifier, or composition does not
pass, the existing parent retry still runs. The verification-only replay has no
native Agent session and is deliberately excluded from client-learning input;
the four actual Agent sessions feed the existing proposal-only learner, whose
output remains `human_gated` and cannot approve itself.

The reviewable decomposition contains six smallest leaves. Formal AI authored
the two contract leaves through Agent CLI (2/6, or 33%), while the release-gate
generalization, regressions, replay checks, and documentation remain
human-reviewed. Reproduce the run after building the release binary and
installing Agent CLI:

```bash
experiments/issue_924_self_authoring/run.sh
```

The runner preserves the dispatch report, every native/replay session, the
Formal AI server trace, proposal-only learning artifact, and the exact authored
contracts. A failed run keeps its temporary workspace so its error remains
available for the next self-development iteration.

## Verification

- `cargo test --test unit specification::self_hosting_metric` exercises fake
  PR claims, matching merge ancestry, ledger recording, target growth,
  regression refusal, and release-pipeline wiring in fixture repositories.
- `cargo test --test unit docs_requirements_issue_924` pins the requirement
  map, contribution protocol, release integration, incremental replay,
  proposal-only learning, and exact Agent CLI-authored contracts.
- `cargo test --test integration issue_991_incremental_dispatch` proves a
  passing child composition cannot be regressed by a redundant parent call.
- `rust-script --test scripts/self-hosting-metric.rs` keeps the standalone
  release script independently executable.

The implementation intentionally leaves the ordinary PR-time share check in
place. The per-release PR floor complements that differential check; it does
not weaken any existing review, CI, promotion, evidence, or metric gate.
