# Self-coding CI evidence and useful-change acceptance

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
   and per-dialog exchanges. Metadata previews are not complete payload capture.
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
5. Use the default Actions token and artifact storage. Optional app/PAT credentials\n   are not used for drafts: their automatic PR events could bypass diff admission.\n   Gist publication and
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
