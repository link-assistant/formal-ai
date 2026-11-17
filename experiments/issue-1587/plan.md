# Issue 1587 work plan

Objective: satisfy every acceptance criterion in PR 1595 on issue-1587-9fa4ea8ba625.

Frozen substantive reasoning units (8): adapter root cause; constraint formalization and precedence; language composition; format/label/length composition and refusal; user-fact retrieval; request isolation; runtime parity; regression and CI evidence.
Formal AI reasoning attempts, complete failures and external bootstraps are recorded separately; supplied commands or code earn zero independent credit.

Requirements and verification:
- [x] Formalize every system/developer message as request-local constraints with message provenance.
- [x] Keep earlier user statements as dialogue facts; last user message remains the task.
- [x] Apply language and format, label and length constraints, or explicitly name unmet constraints.
- [x] All four original examples plus held-out variants pass exact-answer JavaScript regressions (24 fixtures).
- [x] thinking_steps exposes constraints and their disposition.
- [x] Preserve issue 1559 isolation under repeated and concurrent JavaScript requests.
- [x] Add JavaScript and Rust tests using a shared fixture; regenerate TypeScript.
- [ ] Verify native execution: blocked by the existing main-only native-test policy; do not merge to obtain evidence.
- [ ] Update release fragment, case study and PR description; review full diff.
- [ ] Collect current-head CI failures, fix the complete batch, check local gates, push once per round.
- [ ] Verify current-head CI and clean status; mark PR ready.

Initial evidence: branch had no implementation, all recent workflows required approval without executing tests. Current main merged before drafting, preserving issue 1588 changes.

Implementation drafted in both runtimes; JavaScript original/held-out, protocol,
memory and concurrent regressions pass. The unchanged #1559 suite passes 10/10.
Native execution and current-head CI remain to verify. The complete 138-gate
local run collected seven failures. Missing tools/dependencies and structural
notation literals were corrected and rechecked. The dependency audit additionally
reports GHSA-fc5q-39xc-vjpf in unchanged desktop/VS Code lockfiles; the registry
and advisory list no patched picomatch release (latest 4.0.7). The gate remains
closed; no advisory suppression or unreviewed replacement is introduced.

Final local run: 136/138 passed or skipped; the log-filename abbreviation was
corrected and its gate rechecked successfully. Only the dependency audit remains
red. The eight-unit reasoning inventory produced 1/8 attempts (12.5%) and 0/8
independently accepted units (0%). Implementation and verification evidence were
committed before pushing; native tests and green current-head CI remain pending.
