# Python runtime observations

Head `624812120` passed 24 hosted workflows but skipped five Python cases because the runner runtime could not parse the unchanged upstream syntax. This is a coverage defect, not hosted Python acceptance. Full prior hosted logs are under `evidence/ci/624812120/`.

The regression fails before the fix when hosted execution succeeds with four skips. Runtime provisioning now follows the actual longest-first shard selection, pins the upstream Python 3.15.2 interpreter only on the owning shard, and reuses the selected test file for execution. Missing or old Python fails on GitHub Actions; optional local skips remain available. Tests execute both owning and non-owning branches for the two- and eight-shard profiles, with finite child/test budgets.

The final maintained local entrypoint passes 31 tests without skips. Fast profile and default-branch policy checks pass. A focused checkout check and final staged file gates are recorded separately. The preserved draft failures, reporter-context correction, single unchanged VM-timeout retry, and shell-tool limitation are indexed in `evidence.json`. No timeout, benchmark floor, native-build policy, pinned fixture source or upstream implementation was relaxed. Hosted verification of the new head remains required; broader issue acceptance remains unfinished.
