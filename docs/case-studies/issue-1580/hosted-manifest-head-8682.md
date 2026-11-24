# Terminal hosted manifest admission observations

The exact 8682a89a84d391d987a967b5764a27c8d1d0c457 cohort completed on 2026-11-23: 33 workflows, 31 successes, one failure and one explicit skip. The CI/CD Pipeline finished at 04:49:38 UTC. Its native producer and all ten native behavior groups passed. Layered CI passed after the preceding maintained TypeScript projection repair.

The [complete evidence](../../../experiments/issue-1580/evidence/checks/hosted-manifest-head-8682/manifest.json) retains the original whole Pipeline and Layered streams, all terminal job metadata, polling observations, 585 diagnostic candidates with bounded classifications and the original rerun-controller refusal. Archive members are verified by byte count and SHA-256. The complete Pipeline stream has 78,589 lines; the separate 3,165-line admission stream was previously read in full and remains in the [manifest repair evidence](native-admission-manifests-20261123.md). This is targeted failure review, not a claim of literal review of every successful whole-run line.

## The two actionable admission failures

The native admission job failed its isolated-regression step. Whole-Pipeline lines 67,276 to 67,280 show that the checked npm inventory omitted a dependency in a compact JSON manifest. Lines 67,284 to 67,286 show that the tracked active harness manifest was unreadable in the sparse checkout. Lines 67,463 to 67,474 report 283 passing assertions, these two failures and the expected Cargo exit 101. The final Pipeline Status failure at lines 78,585 to 78,588 propagates this same admission failure; it is not another implementation cause.

The local repairs parse actual JSON object members and rewrite only selected dependency value spans. Both admission checkouts now retain the harness manifest and recursively restore release-registry children after the experiments exclusion. A real Git sparse-checkout control fails against the original pattern and the intermediate harness-only pattern, then passes against the complete repair. Six additional native parser controls require execution on the next hosted head.

Ten real source-size warnings at lines 2,438 to 2,456 and one unknown report-mode release credential remain recorded and unqualified. Printed evidence paths, echoed error branches, zero cache failure counters, successful tests with failure-related names and intentional negative-control diagnostics are not additional failed jobs. Their classifications preserve the original lines; they do not establish that every warning or original acceptance criterion is resolved.

## Rerun boundary

The controller waited for this exact original cohort, observed its failure, and stopped before issuing a full or consumer-only rerun. No artifact-rerun qualification is claimed for this source. The controller error and full input cohort remain preserved. Hosted execution of the repaired parser, subsequent reruns, aggregate production review, practice qualification and the other original criteria remain unfinished.

PR: https://github.com/link-assistant/formal-ai/pull/1581.
