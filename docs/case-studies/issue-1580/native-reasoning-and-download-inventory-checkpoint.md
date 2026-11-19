# Native reasoning replay and complete download inventory

Published `070b9748` completes all 23 workflows: 22 succeed and the
[CI/CD Pipeline](https://github.com/link-assistant/formal-ai/actions/runs/44421974106)
fails. Its [native job](https://github.com/link-assistant/formal-ai/actions/runs/44421974106/job/135217168022)
passes lint, 99 Rust cases and four native HTTP cases. All 18 scheduling
contracts pass, including the unchanged clock contract's 27 event fixtures.
The restored experiment shell inputs make the required-install contract pass.
The complete sweep exposes one existing download without an inventory entry
or retry flags, in `experiments/issue_1582/continuation-local-tools.sh`.
Native lines 1708–1722 identify this sole failed test; line 1715 contains the
actual curl command. The downstream status job reports that native failure.

All 68 collection payload hashes are verified. The full failed workflow
(23,626 lines) and native job (2,086 lines) are retained. Native spans
1620–1785 and 1810–1990 and the complete 215-line downstream status log are
literally read. Full native and workflow literal review remains unfinished.
The strict original-panic replay union stays 16/53: 37 lack a qualifying
published replay and three original budget failures remain open. A current
matching test name does not qualify a changed complete original test file.

The local complete shell sweep finds 17 remote curl invocations in 382
checked source scripts and exactly one unregistered download. That existing
ShellCheck command now has three retries, a two-second delay, retry-all-errors,
a 15-second connection deadline and a 60-second per-transfer deadline. Its
exact one-invocation count joins the native required-retry inventory. Neither
native completeness assertion nor the minimum sweep count changes. A new
JavaScript regression consumes the native tables and checks available tracked
shell sources before another Rust run: the policy file records 31 passes and
one matching failure before the fix, then 32 passes afterward. It performs no
network requests. Native execution of this repair remains pending.

The new isolated `reasoning_contracts` target includes the existing 17-test
reasoning-loop module and 11-test interpretation/formalization module. The
reasoning-loop source is byte-identical to original failing head `015a6831`,
including step-six candidate validation. The formalization module retains the
previously documented offline repeatability correction and is not byte-identical
to that original head. Adding the carrier changes no assertion. Native admission
includes both consumed modules and the carrier; the bounded focused workflow
runs all 28 with its existing serial, no-fail-fast command. Hosted execution
remains pending. The first local policy baseline has 30 passes and one admission
failure. An intermediate draft fixes admission but still fails the exact command
inventory; both failures remain archived. Correcting that inventory yields
31 policy passes before adding the separate download regression.

Final local validation passes 44 tests in three JavaScript files: 32 policy,
four candidate-validation and eight focused-selector cases. Both changed Rust
files pass 2024 formatting; the shell script passes syntax checking. All eight
affected gates pass. Every focused and aggregate log is read and hashed; all
eight detailed gate payloads are retained and hashed without claiming complete
literal review. No local Rust build runs, timeout rises or ceiling changes.

The previous publication receipt preserves 30 commands and 60 verified output
payloads. Fresh feedback matches the preceding snapshot: one issue comment,
nine PR conversation comments, no inline comments and no reviews. Java
historical pages 143–145 are read fully without truncation, reaching continuous
145/254 pages. The partial archived PyPI record reaches piece 42/145; the
8,766 complete source lines and 123 complete files do not increase. Pages
146–254 and wider template semantic applicability remain unfinished.

All broader frozen acceptance criteria remain open in the work plan, including
complete calendar semantics and CLI fidelity, the remaining original failures,
benchmark floors, source and passage provenance, dependencies, delivery and
all-template applicability. Supplied commands, code, source reading and reports
earn zero independent authorship credit: accepted reasoning remains 0/20.
PR #1581 stays draft and unmerged; the issue is unfinished.
