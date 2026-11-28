# Latest hosted JavaScript scheduling observations

[Layered CI run 44992280972](https://github.com/link-assistant/formal-ai/actions/runs/44992280972)
ran commit `754fd314fd3c08152b19f9f5c51a87ea23fa6940`.
Its complete log contains 913 distinct file completion observations, including
the failed historical Agent recipe file. Groups 2 and 4 emitted the unchanged
270-second warning (raw lines 6333 and 22108). The 360-second execution budget
and the eight shards are unchanged.

Importing these observations raises 32 underestimated scheduling weights and
lowers none. The retained four-worker model now predicts a maximum shard cost
of 237.237 seconds. This is a model, not hosted execution proof. Earlier models
remain covered by the same regression; all six scheduling replay cases pass.
The generic CI speed tests also pass (26 cases). No test is removed or repeated.

The fixture retains both warning records and the failed file's measured cost.
Complete file observation means its process finished, not that its tests passed.
The recipe assertion is investigated separately; balancing its measured cost
does not repair its result. The evidence directory preserves the original table,
updated model, command, intermediate model output and final test logs with hashes.
All supplied changes and experiments receive zero independent authorship credit.
