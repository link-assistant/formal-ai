# The same default JavaScript suite before merge

Pull requests previously selected a smaller fast inventory and set
FAST_PR_PROFILE, which also skipped individual tests inside selected files.
PRs, feature-branch check dispatches and main now select all 902 default files
through the same eight-shard planner and ordered runner. The optional fast
inventory remains a local tool. Existing 360-second budgets, 75% warnings,
longest-first planning, continued failure reporting and native/release policy
stay in force. The planner installs Python only for the shard that needs it.

The new regression fails before repair and passes afterward. It executes the
actual YAML selector for every profile and shard, proving complete partition
coverage, and runs three physical temporary suites to prove two failures remain
visible while a later passing file finishes. Nineteen production-profile tests,
all 145 default-branch policy tests, two Python runtime tests, 25 CI-speed tests
and the final maintained suite pass. An existing checkout expectation lagged
the already repaired real fixture selection; its initial failure and full
successful retry remain retained. The first complete 141-gate run found three
procedure extraction failures, repaired separately with successful retries;
it is not relabeled as a clean full replay. No full 902-file local run or native
build is claimed. Hosted execution of this changed selection remains pending.

An actual planner probe finds no missing, repeated or extra files and predicts
a 264.7-second maximum at four workers. These historical duration estimates are
planning evidence, not actual execution. Full issue acceptance remains open.

The refreshed Hive Mind guidance has 656 lines, Git blob
8a80627a77f3e92aa6ab025d8f1332f166a630bd and SHA256
a16ab78dad9aa28db8c9c03d3d16203c33928ce2820a75904e286122e6296931.
Every line was read. The refresh correctly detected changes from the previous
582-line capture. This batch adopts its same-full-default-suite rule. New
workspace graph, ratchet, event/cancellation and Swift guidance still need an
explicit applicability review; no blanket adoption is claimed. Supplied work
receives zero independent authorship or reasoning credit.

Evidence: `experiments/issue-1580/evidence/checks/full-default-javascript-profile/`.
