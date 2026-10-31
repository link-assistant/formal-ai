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

## Default-branch build policy batch (2026-10-31)

The branch `ci/default-branch-builds-20261031` follows released v0.355.26. It
closes legacy full-PR build overrides, including branch dispatch and composite
binary-builder paths; Pages, native, WASM, image and application compilation are
main-only (or independently authorized main release consumers). PR admission
uses bounded JavaScript Git reads and fails closed on missing merge/base evidence.
The complete diff includes deletions and source-to-doc renames. Version checks
retain automated-release branch handling and reject manual package version edits.

Selected offline JavaScript tests use one PR runner; main keeps its eight-shard
plan. Browser E2E and box-language integration remain available with
FORMAL_AI_FULL_INTEGRATION=run on main. The independent cold full-container build
remains available with FORMAL_AI_COLD_CONTAINER_REBUILD=run, outside the normal
publication dependency chain. Real native image construction/runtime checks and
source/producer/attempt/byte validation remain selected. Cargo package creation
and staged publishing avoid recompiling an already authenticated build.

Local validation uses individual JavaScript files, actionlint without subprocess
linters, generated workflow checks, tracked-change size checks and policy gates.
No Rust compilation/tests, application builds, containers or public archives run
locally. Production timing and remaining cross-service publication limits are
recorded in default-branch-builds.md. CI and the resulting release must qualify
this batch; local fixture success grants no production-build credit.

PR #1478 first-head CI passed the selected JavaScript group (77-second test step)
and reported one fast-source readability failure for a long test regexp. The
follow-up splits that regexp and narrows workflow-lint checkout to .github plus
its canary fixture after observing a 126-second full checkout. Recheck the exact
new head; first-head passing jobs do not qualify a later revision.

## Main repair after PR #1478

Main source 13dda883c630e9259ed356e0044cf06fe7e13d17 is blocked before release
publication. Follow-up branch: fix/main-dependency-audit-20261031.

- CI/CD Pipeline run 41181550676, lint lane 1 job 124480192498: registry advisories
  updated on October 30 now reject minimatch below 11.0.1 and glob 9 through
  14.0.1. Desktop, VS Code and browser-test tooling are affected. Retain the VSCE
  Azure security pin. A small private local package delegates pattern matching
  to upstream minimatch 11.0.1 and preserves the callable CommonJS API used by
  older packagers/serve-handler. Modern ESM exports pass through unchanged.
  Each consumer directly owns the local dependency and overrides transitive
  requests through that root reference. install-links=true copies the facade
  into node_modules; it cannot depend on a sibling's installed node_modules.
  Glob 9+ resolves to patched 14.x; the unaffected legacy glob 7 API is retained.
- Update only affected lock entries through npm's resolver. All three audit
  verdicts are zero vulnerabilities. npm-ci dry runs validate their resolution;
  no application dependencies or build scripts ran locally. Install only the
  facade's four tiny JS dependencies for its focused CJS/ESM unit test.
- Layered CI run 41181550510: shard 5's coverage producer fixture omitted the new
  GitHub event/ref context; shard 8's checkout fixture and shard 1's generated
  transition fixture expected the old sparse source closure. Shard 7 expected
  the Rust detector command to be a single YAML run scalar. Update those fixture
  expectations while preserving refusal/mutation tests. The motto must follow
  the vision title immediately; keep the build policy after it.
- Add every observed failing test and the real matcher compatibility test to
  the selected fast PR inventory, including the isolated dependency install.
  Native compilation and main-only production checks remain in GitHub CI.

Advisory sources:
[matcher amplification](https://github.com/advisories/GHSA-g3hv-75wg-m249),
[glob cancellation](https://github.com/advisories/GHSA-3g3m-2fjr-6pw4).

Root package.json and bun.lock remain unchanged. Automatic approval review
rejected a broader root-manifest proposal; the repair uses only the affected
consumers and the isolated compatibility package instead.
