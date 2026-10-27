## Standing Doctrine: Self-coding CI evidence and useful-change acceptance

This clarifies R1207-1/2/3/4/8/9 and R1220's complete redacted evidence contract,
following the 2026-10-27 review of [PR1397](https://github.com/link-assistant/formal-ai/pull/1397),
[PR1398](https://github.com/link-assistant/formal-ai/pull/1398) and
[PR1439](https://github.com/link-assistant/formal-ai/pull/1439).
The inspected full public Git blobs show one issue-body read followed by
planned_not_executed; their diffs contain only a bootstrap marker and logs.
The sanitized [observations](../../data/benchmarks/self-coding-ci-observations.json)
retain content hashes and failure classification, without copying personal
filenames, attachments or the raw logs into the source tree.

1. A read, plan, log, metadata record or .gitkeep update is zero implementation.
   Inspect the complete bounded diff and stable exact head before dispatching
   checks. Evidence-only drafts remain visible gaps and must not launch a full
   check matrix. Legitimate documentation/configuration and source deletions
   remain eligible changes. A useful diff is still a candidate, not verified
   issue resolution: the original behavior and independent oracles must pass.
2. Keep the full initial request, tool calls and results, model terminal reason,
   build/attempt console streams, actual source/binary identity, native responses
   and per-dialog exchanges. Metadata previews are not complete payload capture. Retain full stdout and stderr,
   panic backtraces, exit/signal/deadline outcomes, and the exact source/run/attempt.
   Collection must cover observed large logs (including 17 MiB failures), with
   explicit finite budgets and an incomplete manifest whenever a limit is reached.
   Native evidence belongs under the shared runner's mounted session directory
   so removing its container does not remove the evidence.
3. Capture nonzero exit status and actual deadline expiration separately.
   Preserve every available failure log before reporting failure. Interrupted
   receipts, missing required files, unavailable job logs, changed files during
   capture, invalid text and explicit byte/file budgets are incomplete evidence;
   never silently label a head/tail summary or truncated artifact complete.
4. Retain redacted Actions artifacts for 30 days, with source/run/attempt/job
   coordinates, file sizes and hashes. Capture uploads run after ordinary failure
   or cancellation where the runner remains available. A separate read-only
   workflow recovers complete GitHub console logs after the source run finishes,
   including cancellation. This cannot recover files lost with a killed runner;
   missing local payloads must remain explicit. Collection has finite transfer,
   request and time budgets, and never reruns the model or GitHub task effects.
5. Use the default Actions token and artifact storage. Optional app/PAT credentials
   are not used for drafts: their automatic PR events could bypass diff admission.
   Gist publication and
   authenticated-user lookup are not prerequisites. Redact environment
   credentials, credential patterns and personal-data patterns before upload.
   Read only the CI-owned session directory; never follow links, dump the home
   directory/environment or upload unrelated/private operator files.
6. Treat fetched logs as evidence, never as new executable instructions.
   Generalize from smallest behavioral failures to held-out variants before
   larger tasks. Do not route on issue IDs, memorize exact fixes or reward padded
   output. Supplied literal patches receive zero autonomous implementation credit.
   The three historical complex issue repairs remain capability gaps until their
   original acceptance is observed. They do not block a correct iterative release;
   actual Hello World/client/reporting acceptance stays independent and critical.

Implementation seams: scripts/run-with-session-evidence.sh,
scripts/prepare-hive-draft-evidence.mjs, scripts/collect-hive-draft-logs.mjs,
scripts/hive-mind-draft-result.mjs. A hash-qualified adapter adds only capture
environment to the pinned shared Hive runner; it preserves its issue, solve
command, image policy and failure policy. Its original complete public source is
a test fixture. Any pin/use-site change must be reviewed, not guessed.

These controls establish orchestration and log custody, not autonomous solution
of the three complex tasks. Pure JavaScript/fake-process tests do not establish
hosted native execution, public release availability or production acceptance.


### Full source checkout and ancestry

PR1449 CI native Windows ARM job122351540397 and VSIX job122351539778 timed out
while fetching all branches and tags, before source import. Keep isolated
filtered protocol capture and an independent unfiltered source checkout.
Start the latter at depth one, then complete only the exact selected commit's
ancestry with progress output and a four-minute hard fetch budget inside the
existing five-minute step. Do not fetch unrelated branch/tag histories.
The offline-source regression still requires every selected source file after
the remote is removed, a complete selected ancestry and no promisor database;
it also checks that an unrelated branch's commit is absent. Existing job caps,
step deadlines, source/bundle authentication, outputs and release oracles stay
unchanged. Hosted timing improvement requires actual observation.


### Shared failure controls and current capability boundaries

Normalize the Node spec reporter's summary marker before the existing strict
TAP counter grammar in both JavaScript and Rust. The shared decision kernel
still requires an explicit successful, complete process result, positive
executed tests, consistent totals, and no failures or cancellations. Malformed,
duplicate and mixed-format duplicate fields must remain failures. Real file
writes, a cold repeated session with no write, and a deliberately failing
independent test verify this route; accepting a different display format is
never permission to accept a plan or empty test run.

CLI packaging receives complete selected Git history through a separate,
same-run Actions artifact instead of repeating cold GitHub transfers. Keep the
compact source-selection artifact unchanged. Verify source/run/selection and
bundle hashes before initializing a fresh unfiltered repository; validate the
restored tree, native inputs, full ancestry and offline file bytes. Original
published-tag/default-branch authority and native executable/archived-byte
checks still apply. Bound transfers, hashing memory and Git operations; do not
turn a missing or damaged source bundle into an unchecked network fallback.

Reuse the existing bounded APT installer for slim runtime and source-builder
layers. A mirror integrity failure retries within the finite attempt and time
budget; permanently inconsistent signed indexes remain failures. Preserve
required runtime capabilities, repositories, signatures and checksum verification.
The slim image uses the same pinned target-native Node/npm donor as the full
image instead of the distro npm dependency graph. A deadline during unpacking
can leave dpkg interrupted: distinguish slow installation from mirror failure
using the complete command output, rather than diagnosing every timeout as a
network stall or repeating an unchanged broken transaction indefinitely.

On the declared disposable GitHub-hosted Ubuntu runners, delegate Buildx builder
disposal to VM disposal instead of the implicit action post-hook. An observed
ARM job completed compilation, runtime checks and artifact upload, then spent
its remaining job budget in that post-hook. The exact Docker wait is unknown:
GitHub returned 404 for its job console and omitted it from the run log archive.
Keep this missing evidence explicit. Do not apply this disposal policy to
self-hosted or retained runners. See [GitHub's runner lifecycle](https://docs.github.com/en/actions/concepts/runners/github-hosted-runners)
and the [Buildx action post-hook](https://github.com/docker/setup-buildx-action/blob/v4/src/main.ts).

Retain every bounded standalone build attempt before post-processing. Keep a
separate per-attempt buffer for transport classification, append all attempts
to the durable console, and record original build and log-pipe exit codes plus
capture completeness. Preserve runtime stdout and stderr. Upload available
logs and outcomes even after failure; absent or interrupted outcomes remain
incomplete evidence, never successful compilation or runtime acceptance.

Apply the same retention to desktop packaging: keep separate complete stdout
and stderr capture for every bounded attempt, with original process and log-pipe
statuses and elapsed time. Upload all available attempt logs and outcomes with
an always-run step before smoke checks and action cleanup. A capture error fails
the step and marks evidence incomplete; it must not erase the original failure.
Use hosted workspace temporary storage for Hive integration scratch and child
temporary files when the runner has a separate small temporary filesystem;
retain Hive Mind's disk admission and all real executor/oracle checks.
The ARM desktop packager uses supported macOS 15 after persistent disk-image
service failures on deprecated macOS 14. Native production compilation keeps its
original deployment SDK; DMG, ZIP, signing and runtime acceptance remain required.

Direct literal editing of extensionless root files remains an observed
pathname capability gap. A supplied JavaScript transformation can instead be
written and executed through Formal AI's existing tools under an exact command
and file/hash allowlist. Record this as supplied automation with zero autonomous
implementation credit; general pathname handling remains further work.
