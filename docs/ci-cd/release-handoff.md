# Merge handoff during production delivery

A green PR is necessary for a merge, but it does not mean an active production
release is ready for main to advance. Keep main stable through publication and
verify its downstream consumers before merging routine changes. Version
preparation and a successful compiler invocation are intermediate steps.

## Before merging

1. Identify the exact active main producer, its attempt, tested commit and
   prepared version child. Check jobs directly; an aggregate pending status
   does not establish failure, success or publication authority.
2. Continue drafting and testing fixes while CI runs. Hold routine merges while
   that producer still compiles, verifies or publishes. The repository write
   lock serializes CI writers; it does not serialize a maintainer's PR merge.
3. Verify the main pipeline and its publication jobs, then desktop/native,
   browser, source-network, extension and public-download consumers. Distinguish
   source preparation, actual publication and independent download verification.
4. Merge the reviewed green PR, record the merge identity, and monitor its new
   main run. Documentation-only automatic changes retain their lightweight CI
   and no-release policy.

If a critical fix must supersede a failed producer, preserve its logs and let
the new main run release the fix. A freshness refusal is a safety outcome, not
permission to publish the earlier binary under a newer source identity. Do not
remove the check, substitute a tree or receipt, or retry the obsolete run.
Only a proven transient transport failure warrants a bounded retry.

## Ambiguous GitHub release-create responses

The [v0.355.24 create job](https://github.com/link-assistant/formal-ai/actions/runs/41045510144/job/124113910176)
passed source and image validation but the GitHub CLI returned
`unexpected end of JSON input`. The log does not establish the upstream reason
for that truncated response. The original publisher immediately failed, leaving
the already published crate and container images without a GitHub release.

The shared invocation now supervises the unchanged Rust publisher through a
bounded JavaScript recovery loop. After this specific ambiguous transport error,
read the exact tag through the authenticated API. An existing stable release
counts only when its repository, tag and tag commit match the selected source.
Only an authenticated 404 authorizes another write. Stop after three attempts,
and stop immediately on authorization/validation errors, unknown observations,
subprocess timeouts or incomplete cleanup. Log each attempt and observation with
the selected credential redacted. A deferred or failed observation grants no
publication credit. Source/main freshness guards remain unchanged.

The captured error and response-loss scenarios are replayed in the fast
JavaScript inventory without creating real releases or invoking Rust locally.

Do not rerun only failed publication jobs against artifacts sealed by an earlier
run attempt. The v0.355.24 failed-job retry advanced the consumer to attempt 2
while its source descriptor still declared attempt 1; the transfer guard
correctly refused it. Retain that attempt boundary. Recover transient writes
inside the original attempt, or let a fresh main run produce new artifacts
after the repair merges. Never relabel an earlier artifact as a new attempt.

## Publication-observer server failures

The [v0.355.25 self-authoring job](https://github.com/link-assistant/formal-ai/actions/runs/41084627272/job/124217768379)
failed before authoring when its single 100-job request received HTTP 504.
The shared JSON observer now bounds each read, retries only clean HTTP
502/503/504 responses at most three times, and redacts the selected credential
in diagnostics. Exact-attempt job inventories use twenty-job pages and still
refuse missing, duplicate, drifting, oversized or foreign observations.
The release writer recognizes that same upstream error class, but follows its
existing exact-tag/source reconciliation before any further write. A failed
or unavailable reconciliation does not authorize another creation attempt.

Release delivery takes priority over this optional post-publication canary's
access to GitHub. If those server-error retries exhaust, report the observation
as **Deferred**, keep authoring inactive, and award no self-development credit.
This is temporary operational debt for external observation availability, not
evidence of authored or reviewed work. Authorization errors, invalid JSON,
incomplete cleanup, timeouts and provenance violations remain failures. Core
coding/QA checks and production artifact verification remain mandatory.

PRs replay the captured JavaScript cases and do not allocate the live authoring
job. Explicit live routes remain available. Optional App credentials request
Actions read and workflow-file write permissions; checkout must not retain a
default-token header that would shadow the selected author's authenticated
remote. No additional secret is mandatory.
Configure GitHub CLI's selected-token credential helper before source fetches
and authoring. The credential policy verifies that explicit path; ordinary
direct Git writers retain their checkout credential.

## Browser test server preparation

The [main browser shard](https://github.com/link-assistant/formal-ai/actions/runs/41116576030/job/124273089332)
timed out before running tests: Playwright rebuilt the entire web distribution
inside its fifteen-second server-start window, although CI had already built
and synchronized the assets. CI now explicitly selects those prepared assets
only after both preparation steps succeed. Standalone runs retain preparation.
Allow a bounded two-minute startup window and pipe both server output streams
into CI logs. Preserve the browser tests, assertion limits and suite deadline.
Fast JavaScript tests exercise both configuration paths without launching a
browser, building products or running Rust locally.

The older R105 mirror policy must inspect the evaluated Playwright startup
command, including computed configuration, rather than require contiguous
source text. Keep that regression alongside the startup checks in the fast
PR inventory. Shared configuration evaluation starts no server or browser.

The [hosted compiler discovery job](https://github.com/link-assistant/formal-ai/actions/runs/41116575751/job/124273018371)
also observed a TLS handshake timeout before downloading an archive. The shared
installer retries transient connection establishment at most three times with
thirty-second request limits and one/two-second backoff. It never retries a
partially read archive or weakens digest, redirect, workspace or process grants.
Authorization, certificate and policy failures stop immediately. Each attempt
logs a sanitized URL and error class. CI replays this captured failure and the
retry exhaustion/refusal controls using fake transport and clocks; it retains
the real hosted install/compile/Hello World oracles. Run those Python controls
only in CI, with JavaScript transaction tests available locally.

## Observed failure: release v0.355.22

The [production compile job](https://github.com/link-assistant/formal-ai/actions/runs/40986730103/job/123853286634)
compiled successfully in seven minutes. Its post-compile receipt validation
then failed with `prepared release must be the independently current main source`.
The prepared child was `15483855442dbbfc0e061745e0f5aa0de0a97ef1`, but
[PR #1470](https://github.com/link-assistant/formal-ai/pull/1470) had already
advanced main to `5d2f63a4a40b3d758c9b20cb1e9d33b8f98e9bbd`.

The operator waited for `Auto Release / auto prepare-source` to succeed, then
merged before the post-compile check in `scripts/prepared-release-binary.mjs`.
Preparation success was mistaken for completion of the freshness-sensitive
publication boundary. The check correctly refused stale publication; neither
Rust compilation nor the earlier Windows ARM checkout repair failed here.

The operational correction is to hold subsequent merges through publication
and consumer verification, keep drafting in parallel, and let the fresh main
producer deliver the accumulated fixes. Existing fail-closed source checks and
their stale-source regressions remain unchanged. These documentation changes
are manual and claim no Formal AI self-development credit.
