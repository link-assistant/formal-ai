# Whole native-producer rerun and artifact retention

Pipeline [45162493977, attempt 2](https://github.com/link-assistant/formal-ai/actions/runs/45162493977/attempts/2)
finishes successfully on bc08222b5a5195dc9992c0459bde2b6369377b9f.
The single whole-run request was made at 13:18:37 UTC on November 23.
Its 43 jobs resolve to 22 successes and 21 explicit skips. Those skips provide
no execution evidence for main-only behavior or original unmet criteria.

The new native producer succeeds and uploads attempt-2 artifact 14822114800,
with SHA-256 c658669845f1f12f1cbc9c45e453c86cfae046f1314b14fca75f207f92745727.
All ten native consumers select that exact ID and producer attempt 2. Each
consumer records the same downloaded digest and a passing test summary:

| Group | Selected passes |
| --- | ---: |
| coding-1 | 35 |
| coding-2 | 28 |
| reasoning-1 | 125 |
| reasoning-2 | 122 |
| interfaces | 80 |
| benchmarks | 11 |
| hive-1 | 5 |
| hive-2 | 12 |
| hive-3 | 13 |
| hive-4 | 10 |

These 441 passes prove new-producer custody and execution on the baseline.
They do not qualify later local repairs or every original workflow family.

## Retention refusal and evidence

The collector also required the original attempt-1 artifact to remain available.
That assertion fails: artifact 14812694215 disappears from the paginated listing,
and its direct metadata endpoint returns HTTP 404. It was present before the
rerun with a November 24 expiry and a different digest. No local deletion request
was issued. Distinct names containing run attempt therefore do not establish
retention across a whole rerun. The existing [upstream report](https://github.com/actions/upload-artifact/issues/585)
describes the same loss; it is supporting context rather than proof of this run's
cause. Original-archive retention remains unverified and unfinished.

The initial observer stops at that failed assertion. A separate read-only collector
preserves the 404, terminal metadata, complete CLI log and all ten custody/test
witnesses without issuing another rerun. The [sealed receipt](../../../experiments/issue-1580/evidence/checks/native-producer-rerun-20261123/manifest.json)
retains both observations. Every member is read back and hash checked. The
78,743-line log is retained completely; review is targeted to custody, test
summaries and warning candidates, rather than a full literal log review.
Crates.io publication authority still reports unknown in this attempt.

[PR #1581](https://github.com/link-assistant/formal-ai/pull/1581) remains draft.
Supplied work receives zero independent authorship credit. All other unmet issue
criteria remain open.
