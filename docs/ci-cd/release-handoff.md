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
