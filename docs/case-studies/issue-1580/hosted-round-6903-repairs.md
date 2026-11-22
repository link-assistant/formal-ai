# Repair the next complete hosted failure batch

Published source `6903d9bd28ac96f3e73b35d700dce1415c0f5ac6` has three
known failing workflows. The remaining native Hive jobs are still running; this
checkpoint does not claim that the terminal failure inventory is complete.

## Reproductions and repairs

- Workflows run 44978282591: raw line 2688 reports GitHub port 443 connection
  failure after 135797 ms; line 2692 records the three-minute checkout timeout.
  Low-speed transfer settings alone cannot bound a TCP connection. Keep the
  original three-minute acquisition allowance, split into one-minute and
  two-minute checkout attempts, and require an actual successful outcome before
  linting. Preserve identical sparse/read-only checkout inputs and the ten-minute
  job bound. The executable terminal guard covers success, recovered failure,
  exhaustion, skipped work and cancellation. All 161 workflow policy checks pass.
- Layered CI run 44978282529: lines 19348–19488 expose two reference-identity
  assertions after YAML alias expansion. Compare both complete parsed step
  sequences structurally. Both focused files pass while the original SARIF gate,
  diagnostic failure handling and complete audit scope remain asserted.
- The same Layered CI log, lines 25545–25563, exposes the public verifier's stale
  stored hash. Read the actual original Git object
  `101992d84e7b12c843722992050585192dfc3996`: its downstream digest is
  `11977c67b61a050577b42afa0ea0f0f37bc321e03601bd66e1b5e03d3b8f4018`.
  Reverse only the two reviewed artifact adapters, runner pinning and single
  explicit Mac Bash default for that comparison. Assert those exact counts and
  retain the required Mac default. Every other downstream byte stays bound to
  the original source. All 40 focused release-download tests pass.
- Pipeline run 44978282675, native job 137073043040: the actual type/lint step
  passes, while raw lines 2060 and 2122–2127 expose one stale Zizmor action-name
  contract among 195 workflow contracts. Follow the actual shared online runner
  and both ordered personas. Verify its documented official version and immutable
  64-hex digest; the old version test otherwise passed vacuously with no matching
  actions. All six actual JavaScript runner controls pass. Rust formatting passes;
  native execution of the changed contract remains hosted and pending.

The [complete captured streams, procedures, original source and reviewed diffs](../../../experiments/issue-1580/evidence/checks/hosted-round-6903-repairs/manifest.json)
retain byte hashes. Failed intermediate checks and controller assumptions remain
preserved separately from successful production checks. There was no local Rust
compilation or execution. All 142 local gate entries completed: 89 passed and 53
explicitly require native/context evidence. Final TypeScript currency and
selected Rust formatting pass after official projection generation.

The supporting-source regression is documented separately in
[owned-source-batch.md](owned-source-batch.md). All source/diff bytes in this
local repair batch were read by the coordinator; the much larger aggregate PR
review remains unfinished. Wider template/practice execution, original strict
acceptance, live serving identity and genuine independent authorship remain
unfinished. Supplied patches and controllers earn zero independent credit.

Pull request: https://github.com/link-assistant/formal-ai/pull/1581.

## Terminal collection of the published source

All 33 workflows are now terminal: 29 succeeded, three failed and the explicit
Issue 1028 scope gate skipped. The complete 106,651 log lines, all 163 check
records and all 29 annotations are retained with original byte hashes in
[the terminal collection](../../../experiments/issue-1580/evidence/ci/6903d9bd-terminal-round/manifest.json).
All ten actual native contract groups succeeded. The failed type/lint job's
compiler step succeeded; its separate workflow-contract step exposed the one
stale Zizmor contract described above. All three actionable workflow failures
have local repairs. This is verification of published `6903d9bd`, not of its
three later implementation commits.

Every annotation has an explicit disposition: seven failure annotations map to
the repaired failures and terminal propagation; one warning preserves genuinely
unknown crates.io publication authority; twenty-one notices retain the explicit
unexecuted heavy paths, successful zero-error cache statistics, macOS capacity
notice and successful deliberate BuildKit outage control. Printed shell bodies,
expected Git failure controls and browser-import negative-control stacks are
not additional failed producer results. Their original full streams remain
available for inspection; no whole-log literal-reading claim is made.

Fresh registry observations on 22 November still show command-stream 1.13.0
requiring Chrono ^0.4.45 and link-calculator 0.24.0 requiring Chrono ^0.4.
Command-stream's source migration report is closed, but that change is not yet
available in its latest published crate. Calculator's migration report remains
open. Both published dependency paths still block removal of the existing
exact-version, self-expiring exception. No source fix or public registry read
is presented as proof of a dependency migration or publication authority.

Current main `82e506f5447cd6c0b96095993107d6d028a890f5` is an ancestor of
the local branch. All issue comments, PR conversation comments, inline comments
and reviews were refreshed; their counts remain 1, 11, 0 and 0. The aggregate
PR review, applicable-practice execution, strict/live acceptance and genuine
authorship criteria remain unfinished. The next published source requires its
own complete hosted verification before any readiness claim.
