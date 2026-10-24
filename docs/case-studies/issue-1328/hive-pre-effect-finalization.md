# Hive failures before an effect starts

After PR #1402 merged as `71be7a07321a4f03e8bda8ab49c91817f746eb21`,
default-branch cleanup run [40061761443](https://github.com/link-assistant/formal-ai/actions/runs/40061761443)
failed for owner [40056943769](https://github.com/link-assistant/formal-ai/actions/runs/40056943769).
The owner used protocol `43671c81673e3fd5040de1830175a2152e7d3a00`.

## Root causes

The Codex job failed its read-only credential preflight with
`missing configured credential for canonical external task repository`.
The repository had neither an `AUTOMATION_TOKEN` secret nor the configured
`AUTOMATION_APP_ID` and `AUTOMATION_APP_PRIVATE_KEY` pair. Only configuration
names were inspected; no secret values were collected. The default repository
token cannot authorize the three canonical external task repositories.
Claude and Agent were cancelled during image loading. All three jobs skipped
the effect claim, immutable claim publication, and actual solve steps.
No actual task effect was started by this owner.

The cleanup finalizer treated every client without a final generated issue URL
as retained, even when GitHub's complete terminal step metadata showed that
every effect gate was skipped. Its redirected JSON preserved the incomplete
receipt, but the visible job log only showed exit code 1.

## Changes and limits

An admitted execution checks credential configuration before build/image jobs
are allocated. Missing configuration fails explicitly; it never becomes a
successful solve or a fallback to the current repository token. The existing
read-only canonical owner/repository, push and workflow-scope preflight remains
mandatory immediately before the effect claim. Presence alone proves no access.

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
require the complete App pair or an explicit token and never echo either.

The operational credential requirement remains unresolved until an authorized
repository operator configures credentials for the canonical external task
repositories. This change does not claim issue #1364's released three-client
acceptance. No Rust compilation, Docker build/pull or product execution was
performed locally.
