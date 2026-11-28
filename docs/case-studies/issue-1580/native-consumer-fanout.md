# Independent native consumer scheduling

The hosted [pipeline 45218149963](https://github.com/link-assistant/formal-ai/actions/runs/45218149963)
on 7ec084a7f produces its shared artifact at 16:42:01 UTC on 2026-11-23.
The retained job snapshot shows four Hive consumers still queued after 18 minutes,
while two other consumers run. Queued jobs have no assigned runner; their
started_at fields alone do not establish execution.

The actual workflow restricts its ten independent consumers with max-parallel: 2.
That policy permits only two simultaneous jobs even when other hosted runners
are available, contrary to CONTRIBUTING.md lines 886–889. It was introduced with
the original bounded-family split, rather than inherited from main. Remove that
matrix restriction; retain the shared producer dependency, all ten groups,
unchanged assertions, twenty-minute job clocks, failure propagation and worker
process limits. GitHub capacity can still cause queues.

The existing workflow regression fails against the original restriction and
passes after removal. The complete native group, checkout, rerun and headroom
files pass 67, four, seven and 32 cases. The broader policy entry passes 179 cases,
including imported controls; those counts are not independent totals. Its first
attempt was interrupted by the Formal AI shell's default sixty-second command
limit. The completed background retry is retained separately. No local Rust
build or native test execution occurs.

All ten affected JavaScript gates pass. Exact-source hosted timing remains pending.
Original observations, source preimages and checks are retained in the
[receipt](../../../experiments/issue-1580/evidence/checks/native-consumer-fanout-20261123/manifest.json).
This repair does not establish original full-suite budgets, artifact retention
or full issue acceptance. Supplied work earns zero independent authorship credit.
[PR #1581](https://github.com/link-assistant/formal-ai/pull/1581) remains draft.
