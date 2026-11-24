# Hosted capacity and Cargo archive targets

The terminal source 5ce441456821446d2fd2b65bd3b51f94a422d9c7 finishes
with 32 successful workflows and one explicit skip. This does not complete
[issue #1580](https://github.com/link-assistant/formal-ai/issues/1580) or qualify
later repairs in [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581).

## Complete cohort and remaining warnings

All 33 available complete CLI streams, paginated job observations and diagnostic
inventories are sealed in
[the terminal receipt](../../../experiments/issue-1580/evidence/checks/terminal-5ce441-cohort-20261123/manifest.json).
The archive contains 117,115 log lines; full literal review is not claimed.
The tested Layered merge is accd42df3b138d547c2d39667cc444ef21e65d70.
All 922 JavaScript owners complete; all ten selected native groups pass 441 tests.

Six actual platform warnings remain in this cohort:

- Layered CI run 45262786254, lines 18516 and 21173: groups 2 and 9 exceed
  270 seconds, 75% of the existing 360-second deadline.
- Pipeline run 45262787106, lines 2328–2329: the immutable historical
  prior-scheduling-floor.lino fixture contains 1,418 lines, above 1,400.
- Security run 45262786494, lines 217–218: OpenSSF scores for debug and
  require-from-string are below three.
- Pipeline run 45262787106, line 2826: crates.io publication authority is unknown.

The two dependency scores and publication authority remain unfinished. These
warnings are not changed into successes by the passing workflow conclusions.

## Minimal finite capacity repair

The new complete fixture retains the run response, 922 file durations, both
warning identities and the prior scheduling floor. The capacity probe compares
nine through sixteen groups against thirteen retained cohorts. It preserves
all historical floors, the unknown-owner fallback, four workers per group,
every test owner, the 30-second runner allowance and both execution limits.

Nine groups reproduce both latest warning conditions: modeled maxima including
the allowance are 329.741 seconds in group 2 and 334.741 in group 9. Ten is the
smallest passing candidate. Its latest maximum is 261.873 seconds and its largest
conservative weighted queue is 323.9 seconds, below the 360-second deadline.
The largest result across all thirteen retained cohorts is 269.317 seconds.
These are historical queue models; a new hosted run must qualify actual runtime.

The capture tool now verifies the job groups against the actual source commit's
workflow matrix, rather than assuming the current group count. Reconstructing
the retained nine-group cohort produces all four fixture files byte for byte.

The historical 66-candidate owner-allowance probe remains independently
reproducible at its original nine groups and twelve captures. Its original
weights and policy remain deployed unchanged. The new capacity probe owns
the separate matrix decision.

Run the small regressions without running the full functional suite:

    node --test --test-concurrency=1 rust/tests/web/ci-speed.test.mjs
    node --test --test-concurrency=1 experiments/issue-1580/javascript-capacity-replay.test.mjs
    node --max-old-space-size=128 experiments/issue-1580/probe-hosted-javascript-capacity.mjs /tmp/capacity-report.json

The oversized historical floor is compressed without changing its decompressed
bytes. The existing test still verifies its original SHA-256,
6c6c67502689aaec907964958db336a8fb8aaa8b0aa0b00fee2bfe8ae785cb76.
The incomplete cohort still records 819 observed and 103 unexecuted owners.

## Package guard uses Cargo's actual archive

The package guard assumed rust/target/package regardless of Cargo configuration.
Five tiny offline Cargo packages reproduce the problem without compiling:
default layout works; absolute and relative CARGO_TARGET_DIR, configured
target-dir and a workspace root produce their archive elsewhere. The old path
can therefore be absent or name a stale archive.

The repair reads target_directory from cargo metadata with the same working
directory and environment as cargo package. It rejects invalid metadata and
failed processes, keeps package failures fatal, and reads the resulting
package/name-version.crate. FORMAL_AI_CI_VERBOSE=1 traces the resolved directory;
default output remains quiet.

    python3 experiments/issue-1580/package-archive-target-probe.py
    rustfmt --edition 2024 --config skip_children=true --check scripts/check-crate-package-size.rs rust/tests/unit/workflow_contracts.rs

The native workflow_contracts target now includes the real guard and its six
tests, including actual offline Cargo layouts, stale-archive controls, malformed
metadata and failed processes. Native compilation and execution are delegated
to CI. The original three guard assertions are retained.

The current upstream template helper has the same hard-coded path, blob
81b027f01b40cc4f256922032aea9f63a25f01d7. It was inspected, not executed.
The shared defect is reported in
[Rust template issue #268](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/268).

## Tool routing evidence has a narrow boundary

The syntactic inventory observes 61 workflows and 49 candidate tool uses.
It cannot prove availability for every branch or interpret all shell behavior.
The new behavioral regression executes the actual detector job shell blocks
with isolated fake tools. Eighteen event/ref cases select the proper JS/native
route; nine missing, disabled or late-installer controls fail as expected.
The maintained checkout owner passes all five cases.

This does not execute the native classifier or qualify publication. The Rust
practice ledger retains the remaining tool-availability and release obligations.
All supplied commands and edits receive zero independent authorship or reasoning
credit. Wider template, semantic, dependency, production and retention criteria
remain unfinished; the PR stays draft and unmerged.

## Local verification

The maintained suites pass 438 observation cases, 184 policy cases, 30 speed
cases, 31 profile cases and five checkout cases. Twelve historical/probe and
four partial-observation controls also pass. The initial profile expectation
still named nine groups; its retained failure is corrected to the measured
ten-group capacity without changing inventory or budget assertions. An invalid
local gate selector is retained with the corrected successful invocation.
Affected workflow, speed, runner, size, readability, changelog, documentation
and requirement-status checks pass. Rust formatting passes; native execution
and exact-source hosted qualification remain pending.
