The dependency checker skips pin integrity verification whenever the declared version is stale. This turns an invalid action pin into a warning-only result on push, schedule and workflow_dispatch.

Source reviewed: `65c686846d7327d8c835a0d30b1a76cc6345cfc6`, `scripts/ci/check-dependencies.mjs` (Git blob `62397dcfdd1b1173b16bcb2e750903066695d4e4`). The latest related merged PR #16 introduced the freshness warning policy; this issue concerns its interaction with integrity checking.

In `checkDependencies`, the version-mismatch branch appends a stale finding and then `continue`s before `verifyPin`. The reporter treats stale-only findings as warnings on non-PR events. Integrity verification must run independently of freshness policy.

Bounded offline reproduction using the actual unchanged source and reporter:

| Lookup result / injected verification result | Verification calls | Push / schedule / dispatch | Pull request |
| --- | ---: | --- | --- |
| Current / invalid pin | 1 | failure | failure |
| Stale / invalid pin | 0 | success | failure (freshness) |

Removing only that `continue` in a disposable source copy invokes verification and fails all four events for invalid pins. Current valid pins remain successful; stale valid pins retain the intended warning-only success outside PRs.

The regression injects lookup and tag-verification results through the checker's existing test seams. It does not claim a real remote tag mismatch, execute hosted workflows, publish anything or modify this repository. It also tests the valid-pin/freshness controls to distinguish an integrity regression from the intended non-PR warning policy.

Executable evidence is in link-assistant/formal-ai PR #1581: `experiments/issue-1580/template-go-result-integrity.test.mjs`, case `Go stale-pin integrity failures become warnings; independent verification preserves failures`. The adjacent fixture retains the original license and source hashes. Run `node --test --test-concurrency=1 experiments/issue-1580/template-go-result-integrity.test.mjs` in that checkout. Its five cases pass locally; the maintained fast-PR entrypoint also passes eleven cases without skips. The complete log/hash index is `experiments/issue-1580/evidence/checks/go-pin-integrity.json`.

Immutable regression evidence: https://github.com/link-assistant/formal-ai/blob/a4053def4bd4b921001fdf84a026ae35deebfc39/experiments/issue-1580/template-go-result-integrity.test.mjs. No upstream fix is claimed.
