# Cold research coverage scheduling

[Main coverage shard 14](https://github.com/link-assistant/formal-ai/actions/runs/40254186002/job/121355235944)
on merge `bb7c745cdf1fba24dadbdccca02950f1d8be7a44` reached its unchanged
1,200-second execution deadline. Ordinary native tests passed. The instrumented
log reported no assertion failure; its unfinished harness prevented complete
coverage, so the aggregate report correctly failed.

The cold held-out TypeScript research case had no recorded scheduling cost.
It shared a process with 63 other default-weight cases, giving the whole batch
a predicted cost of 5.888 seconds. Its warning appeared at
`2026-10-26T02:19:49.3038230Z`, after at least 60 seconds of execution. It had no
completion when the shard expired at `2026-10-26T02:31:46.0959765Z`.
That is a censored wall-time lower bound of 776.792 seconds, not a completed
measurement or a passing result. The log does not isolate CPU contention from
the cold procedure's own computation; engine optimization remains follow-up.

The complete 570,421-byte public log has SHA256
`db1fb9ed1d2494b0ed1b291b4860035cb07a040e056b248e1c0f53cee8ef64d7`.
The declarative fixture preserves all warning/completion observations and the
actual unfinished batch. Scheduling costs are increased from those observations
using the existing native-floor policy. The existing longest-first planner
then prioritizes the cold case in a separate process. A JavaScript regression
checks its isolation and preserves every captured batch peer exactly once.

The shard count, deadline, instrumentation, original assertions, exact-name
completion requirements and coverage thresholds remain unchanged. No additional
test is skipped. New hosted execution is required to verify the scheduling fix.

One pure coverage-job rerun is permitted for the original merge, at most two
attempts including its first failure. It gathers a second timing observation
while the permanent scheduling correction is prepared. Preserve the first
failure and its incomplete status. Never rerun a real-task owner or an immutable
cleanup claim under this exception. A published candidate remains unqualified
until every mandatory current-source check and public verification passes.
