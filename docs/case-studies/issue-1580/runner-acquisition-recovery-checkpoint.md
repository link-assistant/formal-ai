# Conservative hosted runner acquisition recovery

The Go template's acquisition recovery is applicable to this repository's
hosted pipeline; the previous comparison left that reusable practice open.
Adapt it for failed CI/CD Pipeline runs on default-branch push or dispatch,
using trusted default-branch code and no artifacts from the failed run.

Read complete paginated jobs and annotations and require the exact repository,
run, attempt, check-run URL and immutable source. Retry only an empty-step,
unassigned job with GitHub's specific acquisition annotation. Actual test/code
failures, ordinary cancellation, missing API observations and superseded source
do not rerun. A known failing Pipeline Status aggregator may accompany the
acquisition failure. Attempts one and two may rerun failed jobs once; the third
reports exhaustion. The job uses bounded writer authority, a non-cancelling
queue, a ten-minute deadline and a three-minute aggregate API budget.

All 27 cases pass, including a real command with a mocked GitHub executable,
pagination on both inventories, source changes, API errors and attempt limits.
The maintained suite imports these controls and passes all 186 cases. The new
workflow_run observer will become available after merge to the default branch;
no real hosted recovery or complete Go-practice acceptance is claimed here.

Supplied implementation earns zero independent authorship/reasoning credit. The full issue remains unfinished.

PR: https://github.com/link-assistant/formal-ai/pull/1581
