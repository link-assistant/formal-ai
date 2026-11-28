# Controlled dialogue language ownership

Layered CI run 45218148759 at published head 7ec084a7f passes, but its sixth
JavaScript shard warns at 270 seconds and completes in 293 of 360 seconds.
The original controlled-dialogue test file takes 281.117672089 seconds:
English 51.401435097, Spanish 35.080045912, Hindi 118.506209774,
Russian 36.871314912 and Chinese 38.486720364. One indivisible file holds
five sequential language tests; increasing the shard count cannot split it.

Three maintained owners now select fixture languages by sorted index modulo
three: English/Russian, Spanish/Chinese and Hindi. The original fixture loop,
assertions, task-shape deduplication and timeouts are preserved. The original
owner also retains both existing non-language controls without changes.
Future fixture languages are assigned once by the same rule.

The ownership regression registers the actual helper cases with a stub runtime
and checks conservation; it does not claim solver execution. Separate real
solver test runs pass all seven cases across the three owners: four in
89.280 seconds, two in 74.086 seconds and one in 169.455 seconds. These are
local wall times, not hosted child measurements. Scheduling keeps the parent's
284-second bound for each child, explicitly as an aggregate upper bound rather
than an observed child duration. The deployed nine-shard planner must place
these owners separately. All existing budgets and worker counts stay unchanged.

The retained original source fails the bounded-ownership regression before the
repair. Original logs, source conservation checks and local verification are retained
in the [complete repair receipt](../../../experiments/issue-1580/evidence/checks/dialogue-and-fixture-repairs-20261123/manifest.json). Exact hosted child durations remain
pending the next combined push. This repair does not finish issue #1580.

The first historical scheduling replay finds 243.608164402 worker seconds plus
the existing 30-second allowance in a stale eight-shard model. The deployed
workflow already uses nine. The replay now reads and cross-checks the actual
matrix and both planning/execution shard settings. All original observations,
warning thresholds, overhead allowances and scheduling floors are retained.

The first complete local gate sweep finds two failures before the batch push.
A repeated three-line scheduling metadata block increases the notation duplicate
count; retaining one shared explanation and each child's aggregate bound
removes that duplicate without changing either weight. ShellCheck separately
reports SC2097 and SC2098 in the search installer because an environment-prefix
assignment is referenced by another expansion on the same command. Computing
the fallback beforehand preserves existing defaults and explicit overrides.
The actual ShellCheck invocation fails before and passes after; all 18 existing
installer behavior tests pass. The final complete replay passes 89 gates and explicitly skips 54 CI-only
or Rust gates; none fails. The maintained observation entry passes all 432 cases.
