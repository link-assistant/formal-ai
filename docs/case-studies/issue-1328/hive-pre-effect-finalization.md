# Hive failures before an effect starts

After PR #1402 merged as `71be7a07321a4f03e8bda8ab49c91817f746eb21`,
default-branch cleanup run [40061761443](https://github.com/link-assistant/formal-ai/actions/runs/40061761443)
failed for owner [40056943769](https://github.com/link-assistant/formal-ai/actions/runs/40056943769).
The owner used protocol `43671c81673e3fd5040de1830175a2152e7d3a00`.

## Root causes

The Codex job failed its read-only credential preflight with
`missing configured credential for canonical external task repository`.
Neither an `AUTOMATION_TOKEN` secret nor the configured `AUTOMATION_APP_ID`
and `AUTOMATION_APP_PRIVATE_KEY` pair was available to this job. Only repository
configuration names were inspected; organization configuration was not visible
to the local credential. No secret values were collected. The default repository
token cannot authorize the three canonical external task repositories.
Claude and Agent were cancelled during image loading. All three jobs skipped
the effect claim, immutable claim publication, and actual solve steps.
No actual task effect was started by this owner.

The cleanup finalizer treated every client without a final generated issue URL
as retained, even when GitHub's complete terminal step metadata showed that
every effect gate was skipped. Its redirected JSON preserved the incomplete
receipt, but the visible job log only showed exit code 1.

## Changes and limits

PR #1403 initially made optional credential configuration a prerequisite. The
latest user requirement supersedes that decision: absent optional credentials
must use the current repository token, following Hive Mind's own resolver and
orphan-branch fixture approach. Read-only repository identity and push preflight
still run before the effect claim. External tokens retain their external scope
checks; the default token is restricted to the current repository and stages
workflow files under `ci/` for execution by Hive's original `act` transport.

Cleanup recognizes a distinct `not-started` terminal state only when complete
logs contain no final issue URL and complete GitHub job metadata contains one
each of the claim, claim publication and solve gates, in order, all skipped.
Missing, duplicated, reordered, active or executed gates cannot prove this state.
The no-op keeps `cleanupReceipt.complete` false, closes no issue, grants no
delivery credit and retains the immutable claim to prevent repeated execution.
An uncertain write always remains uncertain. Started or ambiguous effects still
require the original full issue custody and closure proof.

The CLI prints state, owner identity and closed/retained counts to stderr while
the full JSON remains in the outcome artifact. It never prints credentials or
task log contents.

## Regression evidence

`rust/tests/fixtures/hive-pre-effect-failure.json` contains the public terminal
step metadata of all three actual owner jobs and SHA-256 identities of their
complete logs. It contains no task inputs, credential values or local data.
Regression tests replay the failure and reject incomplete logs, unexpected issue
URLs and every malformed or executed effect gate. Separate tests preserve
immutable claim quarantine and uncertain-write precedence. Configuration tests
cover default fallback, a complete App pair and an explicit token without
printing credential values.

No additional credentials are required. The fallback uses the pinned Hive task
generator unchanged, and verifies that reversing only the two workflow staging
instructions reproduces each complete original task body hash. Two-pass and
independent compilation/output oracles remain intact. Receipts explicitly
distinguish same-repository acceptance from historical external delivery.
The trusted finalizer closes owned issues and PRs and deletes their branches
after the full owner workflow finishes, including cancellation. It corroborates
complete logs, bot ownership, task content and actual PR branches before writes,
preserves human feedback and never retries uncertain writes. Captured public
Hive source fixtures exercise the real generator and fixture creator without
network effects. Local regressions do not claim released three-client acceptance.
No Rust compilation, Docker build/pull or product execution was performed locally.

## Follow-up release coordination

[PR #1403](https://github.com/link-assistant/formal-ai/pull/1403) carries these
fixes. Its description uses explicit acceptance-pending issue links, as required
by `scripts/check-pull-request-link.rs`; it must not prematurely close #1328 or
#1364. CI reads the description from its triggering event, so a corrected
description must be present before the validation event is created.

Allow the existing guarded release producer to finish before advancing `main`.
`scripts/prepared-release-binary.mjs` and the stage-transfer authority require
the prepared release source to remain the independently current `main` during
publication. Merge the follow-up only after its exact-head CI is green and the
current release has completed, with public Docker metadata verified against its
source, binary and runtime receipts. Do not relax those source checks to overlap
the two merges.
