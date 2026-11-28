# Extracted native module repair

The [4f6b pipeline](https://github.com/link-assistant/formal-ai/actions/runs/45124435265)
failed after the source-warning extraction. Its complete cohort contains 31
successful workflows, one failure and one skip. Layered CI now passes.
The complete pipeline log has 69484 lines. Lines 65693 through 65695
report three E0583 child-module lookup errors; lines 67538 through 67547
report an unused renderer import in the native workflow-contract lint job.
The producer failure skipped all ten native execution groups.

The focused reasoning target loads parent modules through path attributes.
Their implicit child lookup differs from the ordinary module hierarchy.
Explicit child paths retain all existing cases in both hierarchies.
The dependency checker keeps its public renderer through a forwarding function;
a native regression checks every finding field, unknown dependencies and empty
input. No lint allowance or scanner threshold is added.

A bounded declaration-only experiment reproduces the previous focused-module
and unused-import failures, while the ordinary hierarchy passes. Both hierarchies
and the renderer pass with the repair. Missing-child and missing-renderer
controls still fail. These eight tiny metadata compilations are structural
evidence, not full-crate qualification; each has a 1 GiB address-space limit,
16 MiB stack limit and 15 CPU-second limit. No Cargo build or repository Rust
test suite ran locally.

The complete policy file passes all 170 JavaScript tests; the source-warning
regression passes its one test. Nine affected JavaScript gates and formatting
of all five changed Rust files pass. The first gate attempt was interrupted
by the Formal shell wrapper's 60-second limit; its logs are retained alongside
the complete detached rerun, rather than represented as a gate failure.

The [hosted receipt](../../../experiments/issue-1580/evidence/checks/hosted-merged-head-4f6b/manifest.json)
retains whole logs, paginated jobs, observations and the fail-closed rerun
qualifier. Its whole-log scan records zero source-size warnings. The
[repair receipt](../../../experiments/issue-1580/evidence/checks/extracted-native-modules/manifest.json)
binds the source repair at 795e76727e85a945f3df04f9f91c11583908cb54, the original failures,
negative controls, local checks and fresh feedback. All archive members were
read back and checked against their recorded byte lengths and SHA-256 hashes.

Hosted execution of the repaired source, actual artifact reruns, strict style
lint debt, publication authority and all remaining original acceptance criteria
remain unfinished in [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581).
Supplied edits and orchestration earn zero independent authorship credit.
