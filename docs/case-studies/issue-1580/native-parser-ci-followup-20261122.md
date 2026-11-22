# Native parser visibility and complete CI policy replay

Published `a0523f676efdeae358dc473e8b8b3ecf721201ba` has 33 terminal workflows:
30 success, two failure and one explicit integration skip. The failed runs are
[Pipeline 45082117907](https://github.com/link-assistant/formal-ai/actions/runs/45082117907)
and [Layered 45082117155](https://github.com/link-assistant/formal-ai/actions/runs/45082117155).
All failed workflow logs, failed-job metadata and three executed native job
streams are preserved in the accompanying round-trip-verified archive.

The native producer and lint jobs both stop at four E0603 errors: external
consumers name `seed::split_pipe_list`, a private import, instead of the existing
crate-visible `seed::parser::split_pipe_list`. The raw lint stream records these
at lines 1288, 1310, 1331 and 1352. Correct the four production consumers and the
same path in the library regression; do not widen the seed module API. No native
contract group executes on this head because its producer failed. The separate
release-registry job succeeds; it is not evidence that the failed crate compiled.

Layered additionally records three stale workflow-policy checks, the stale
TypeScript projection and the inline parser unit test. The workflow was renamed
and expanded to four independent library/language regression commands, while its
policy checks still selected the old step and expected two commands. Both exact
allowlists now cover the four commands. Six finite shell cases run the actual
workflow body with each individual failure, all failures and all successes;
every sibling executes and the final nonzero status survives. The existing
parser regression moves unchanged in meaning to `rust/tests/unit/` with its
original module filter. The maintained TypeScript generator refreshes four
worker projections from their JavaScript source.

Focused verification passes 242 JavaScript cases: 168 workflow policies, five
source-placement checks, three canonical seed cases, 65 native group controls
and the real TypeScript-source runtime comparison. All six touched native source
and test files pass `rustfmt --check`; TypeScript projection and whitespace checks
also pass. The first local run's two stale step-selector failures and the earlier
formatting/assertion failures remain retained. No local Rust build or execution
occurs. Native compilation, real behavior and both actual artifact rerun paths
remain pending the next hosted head.

Evidence: `experiments/issue-1580/evidence/checks/native-parser-ci-followup/`.
Supplied work earns zero independent Formal AI authorship or accepted reasoning
credit. Broader issue acceptance remains unfinished and PR #1581 stays draft
and unmerged.


The final clean combined source `44771e97373f89d4d582bc384c2fd1f8d69cd83d` completes all 143 maintained
local gate entries: 89 pass, zero fail and 54 explicitly require hosted execution.
The run takes 568.11 seconds, with the same clean commit before
and after. Every per-gate stream and the outcome are round-trip verified under
`experiments/issue-1580/evidence/checks/clean-parser-contract-gates/`.
Hosted native execution and the actual artifact reruns remain pending.
