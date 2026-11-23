# Native waiter contract follow-up

[Current-source native admission](https://github.com/link-assistant/formal-ai/actions/runs/45196552928/job/137770001889)
compiles and lints successfully, but its workflow-contract harness reports
293 passes and one failure at original job-log lines 2148–2150 and 2210.
The failing test still requires the previous boolean registry API. The waiter
now calls the shared tri-state sparse-index/API lookup.

The test now requires that exact crate/version lookup and all three explicit
visibility branches. The configured Docker, release, template-skip and shared
registry assertions remain. Production publication behavior is unchanged.
Rust formatting passes; execution of this corrected native test remains
pending on the next hosted source. No local Rust compilation is performed.

[Complete original job log and exact source](../../../../experiments/issue-1580/evidence/checks/native-registry-contract-followup-20261123/manifest.json)
include the initial CLI download refusal and the successful complete retry.
The 3,170-line log has targeted failure review, not a claim of full literal
review. Issue #1580 remains unfinished and
[PR #1581](https://github.com/link-assistant/formal-ai/pull/1581) stays draft.
