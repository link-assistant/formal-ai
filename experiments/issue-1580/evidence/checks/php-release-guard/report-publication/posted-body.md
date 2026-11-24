The automatic-release job can be admitted after lint, test or build fails.

Audited default head: `3a034b27a56067971ca74dd4c74c7707056b215d`.
In [the actual release workflow](https://github.com/link-foundation/php-ai-driven-development-pipeline-template/blob/3a034b27a56067971ca74dd4c74c7707056b215d/.github/workflows/release.yml#L309),
`auto-release` needs lint, test, build and preflight, but its explicit status
condition checks only preflight success. Status functions override the implicit
`success()` check. A failed test plus successful preflight can therefore admit
version/tag publication even though the eventual terminal observer fails red.

The bounded offline reproduction in formal-ai PR #1581 parses the unmodified,
hash-verified workflow. All nine failure/cancelled/skipped quality-check controls
admit the original guard. Requiring explicit success for lint, test and build
blocks them while retaining a successful main push. Preflight failures, wrong
branch, PR/dispatch events and whole-workflow cancellation remain blocked. The
existing manual-release guard already has the required explicit checks.
Removing only `always()` is insufficient while `cancelled()` remains present.

Repair direction: retain the push/main/preflight/cancellation conditions and add
`needs.lint.result == 'success'`, `needs.test.result == 'success'` and
`needs.build.result == 'success'`, as the manual-release job already does.
Retain unsuccessful-result regression controls in WorkflowPolicyTest.

This is an executed expression-model reproduction, not a claim that a hosted
release was dispatched. It requires no secrets, package installation or build.
Immutable executable reproduction: [five bounded guard tests](https://github.com/link-assistant/formal-ai/blob/fda677ef9f25692b7b3823556f1349839a71f7a3/experiments/issue-1580/template-php-release-integrity.test.mjs).
Run `node --test --test-concurrency=1 experiments/issue-1580/template-php-release-integrity.test.mjs` in that checkout.
[Full five-test and maintained 43-test logs, source identities and hashes](https://github.com/link-assistant/formal-ai/tree/fda677ef9f25692b7b3823556f1349839a71f7a3/experiments/issue-1580/evidence/checks/php-release-guard) are retained.
This evidence was published before this report; no upstream repair is claimed.
Related PRs #13 and #24 and all three comment/review endpoints were read.
The existing failed-Git report #26 concerns a separate boundary.
