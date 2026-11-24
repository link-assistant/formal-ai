# Terminal 7ec0 CI collection

All 33 workflows on published head 7ec084a7fc1b6a909b18a0c6ff9acf5b1f44449f
have finished: 32 succeed and one explicitly skips. The CI/CD Pipeline finishes
at 17:28:07 UTC on 23 November 2026. No failed workflow remains on this head.

The [complete receipt](../../../experiments/issue-1580/evidence/checks/terminal-7ec0-cohort-20261123/manifest.json)
retains 32 complete CLI log streams. The pipeline CLI log endpoint returns
HTTP 503, and three individual-job retries also fail. Its complete available
ZIP is downloaded separately, CRC-checked and mapped to all 22 executed jobs;
21 other jobs explicitly skip. The archive is 2,123,599 bytes. Failed download
attempts remain separate observations and are not counted as complete streams.

The pipeline's ten native groups execute 441 passing tests; native admission
passes 294 controls. Registry verification executes four unit and 46 CLI cases
(50 total). These are scoped native executions, not proof of all original
main-only acceptance criteria. The two-consumer concurrency cap serializes the
native matrix: producer completion to last consumer completion spans 45m57s.
Its separate repair still needs exact-source hosted timing after the next push.

A broad diagnostic search classifies 2,112 candidate lines. Five actual
warnings remain: Layered CI line 23044 reaches the 270-second warning threshold;
Security lines 554 and 555 report OpenSSF scores 2.8 and 2.3; Security line 1178
allows the historical unsafe-libyaml 0.2.11 warning in the issue-1017 experiment;
pipeline member `39_Release Preflight.txt`, line 196, cannot prove crates.io
publication authority through a read endpoint. Commands containing warning
options, listed evidence paths and passing test names remain distinct.

The [dialogue ownership repair](cnl-dialogue-headroom.md) addresses the observed
281-second indivisible language test file. Original diagnostic contexts and
classification decisions are retained; this is not a claim of literal review
of every log line. Full practice adoption, external identity, independent
authorship and all remaining original issue criteria stay unfinished.
PR #1581 remains draft and unmerged.
