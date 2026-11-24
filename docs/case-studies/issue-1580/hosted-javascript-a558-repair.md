# Current hosted JavaScript scheduling costs

Layered CI run 44999044965 at commit `a558f834` completed all eight JavaScript
shards, but shard six took 325 seconds and crossed the unchanged 270-second
warning. Its complete raw log has 913 unique file summaries, including three
failed files. A top-level-only dialogue test emitted its own pass marker without
a `node:test` file summary; missing timing data stays a fallback, never a pass.

The retained-cost regression reproduces a 300.55-second four-worker shard with
the previous weights. Importing only raised measured weights leaves 258.73
seconds, which is insufficient once the observed 13–22 seconds of runner
overhead is included. A second failing control retains that distinction.

The revised weights preserve every prior floor and reserve 20% over exact
retained observations. The reserve is a scheduling allowance, not a measured
duration. A separately found importer defect could erase an existing measurement receipt
when a new timing was larger. The importer now preserves that protected record
and its exact source identity. All 913 exact observations remain unchanged in
the fixture, including
the three failed suites. Eighty-six weights increase and none decrease. The
slowest four-worker replay is 217.55 seconds; the regression also reserves 30
seconds for runner overhead. All seven historical scheduling cases and the
existing CI speed controls pass. Every current suite is assigned exactly once;
the warning stays 270 seconds and the hard limit stays 360 seconds.

The previous table, both failing controls, finite reserve comparison, importer,
complete source log and passing checks are content-addressed in
`experiments/issue-1580/evidence/checks/hosted-javascript-a558-repair/manifest.json`.
This replay is scheduling evidence. It is not hosted execution proof or whole
issue completion. The next hosted head and the wider acceptance remain open in
[PR #1581](https://github.com/link-assistant/formal-ai/pull/1581).
