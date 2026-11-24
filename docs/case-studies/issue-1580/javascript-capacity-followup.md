# Retain the warning from the ten-group deployment

[PR #1581](https://github.com/link-assistant/formal-ai/pull/1581) deployed ten
JavaScript groups at aa84dfc247891313dc5dc20847950e91c9528064. Layered CI
[run 45271423335](https://github.com/link-assistant/formal-ai/actions/runs/45271423335)
passed on tested merge 3c887a7ac57dd0dcbd685cace9b650ca29b8b4ad, but group nine
warned at 270 seconds. Its complete log retains that annotation at line 22,757.
Passing the previous thirteen-cohort replay did not guarantee a warning-free
execution.

The new capture retains all 922 completed owners, all ten successful job
identities, the actual annotation and the unchanged scheduling floor. Several
owners varied materially from the prior run: the Hive work-item consumer
increased from 100.442 to 154.810 seconds and the release-completion producer
from 18.075 to 92.599 seconds. No owner in this run individually exceeded
the warning threshold. The accumulated work, under the retained allocation,
exhausted the warning headroom.

## Bounded reproduction and correction

Run:

    node --test --test-concurrency=1 experiments/issue-1580/javascript-capacity-replay.test.mjs

Adding the fourteenth capture makes the unchanged deployed-matrix assertion
fail: the finite replay now requires eleven groups. Ten groups project
284.882 and 282.221 seconds for groups nine and ten, including the unchanged
30-second runner allowance. Only group nine actually warned; the replay
remains an estimate.

The bounded search still considers exactly eight candidates, nine through
sixteen groups. Eleven is the smallest candidate fitting every retained
cohort, the 270-second warning threshold and the unchanged 360-second
conservative queue deadline. Its latest-cohort maximum including allowance
is 231.509 seconds; the maximum across all fourteen cohorts is 269.317.
The test keeps the previous thirteen-cohort selection of ten as a historical
control, both earlier nine-group warning projections, every owner, exact
partitioning, deterministic ordering and the complete original timing floor.
The deployed matrix and both planning/execution environment counts use eleven.

A separate finite experiment tried a per-owner maximum across every historical
source version. None of its eight candidates fit. Those versions include
indivisible owners subsequently split or repaired; combining their individual
maxima is not an observed current workload. The complete failed output is
retained and that experiment does not select the deployed allocation.

The first local production-profile run also caught its stale ten-group
expectation. Its failure is retained; all four event contexts and the complete
partition assertion are updated to the measured eleven-group allocation.
The rerun then caught the deliberately changed link condition in its original
whole-job payload digest. That failure is also retained. The test now asserts
the exact repaired condition and normalizes only that field for the original
digest; every other job field and every original manifest hash stay unchanged.
The independent branch-policy owner also retained a literal ten-group count;
its original failure and all four corrected PR/main count assertions are retained.

This repair does not extend a timeout, relax a warning, shrink the suite,
reduce concurrency or change a scheduling weight. Exact-source hosted
qualification remains pending. Original issue requirements, including complete
template execution, production delivery, semantic coverage, dependency
criteria, independent authorship and aggregate review, remain unfinished.

Complete outputs and identities: [verified archive receipt](../../../experiments/issue-1580/evidence/checks/javascript-capacity-followup-20261123/manifest.json).

## 24 November hosted warning

[Layered CI 45334688742](https://github.com/link-assistant/formal-ai/actions/runs/45334688742)
ran head `5c6cedff515988659748a54f63d4630af352ef26` and tested merge
`2fe601727eefaa43980796d37b02a168847c62a8`. All eleven jobs and 922 owners
finished. Group 2 crossed 270 seconds at full-log line 3510 and finished
in 275.484 seconds at line 4116.

The fifteen-cohort replay selects twelve groups. At eleven, it projects
292.377 seconds for the observed group 2 and 270.398 seconds for group 9.
At twelve, the new cohort's longest projected group is 235.704 seconds.
The maximum across all cohorts is 269.317 seconds. All projections include the
existing 30-second allowance. The matrix keeps four modeled workers, the
same owner inventory, the same weights, and the 270-second warning and
360-second deadline.

Reproduce with `node --test --test-concurrency=1 experiments/issue-1580/javascript-capacity-replay.test.mjs`.
The prior workflow fails the new matrix assertion. The receipt preserves
all 922 owner events and their original job and log identities.
The revised allocation still needs a hosted run.
