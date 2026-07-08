# Issue 540 Online Research

Research date: 2026-07-08.

## Background Compaction

- RocksDB compaction wiki: https://github.com/facebook/rocksdb/wiki/Compaction
- RocksDB low-priority writes: https://github.com/facebook/rocksdb/wiki/Low-Priority-Write

Useful pattern: storage maintenance is a background job and can be prioritized
below foreground writes. Formal AI mirrors this by making dreaming a pure plan
that can run in the background and by using desktop low-priority process
scheduling where supported.

## Routine Vacuuming

- PostgreSQL routine vacuuming: https://www.postgresql.org/docs/current/routine-vacuuming.html

Useful pattern: the goal is steady-state disk usage, not shrinking every table
to minimum size. Formal AI mirrors this by targeting a configurable free-space
reserve (20% by default) and by freeing only enough reclaimable data to satisfy
that reserve.

## Cooperative Idle Scheduling

- W3C requestIdleCallback: https://www.w3.org/TR/requestidlecallback/
- MDN Background Tasks API: https://developer.mozilla.org/en-US/docs/Web/API/Background_Tasks_API
- MDN requestIdleCallback: https://developer.mozilla.org/en-US/docs/Web/API/Window/requestIdleCallback

Useful pattern: background work should yield to latency-sensitive work. Formal
AI's desktop scheduler delays the first run, repeats infrequently, unrefs timers
and child processes, and keeps the operation plan-only by default.

## Cache Eviction

- Redis key eviction: https://redis.io/docs/latest/develop/reference/eviction/
- Redis Enterprise eviction policy: https://redis.io/docs/latest/operate/rs/databases/memory-performance/eviction-policy/

Useful pattern: eviction is a declared policy over cacheable data, with LRU/LFU
style usage signals under pressure. Formal AI applies this only to data it can
refetch or recompute and uses recalculated event references as the current usage
signal.

## Local Prior Art

- `src/memory.rs`: append-only memory with explicit purge/reset maintenance.
- `src/main.rs`: CLI confirmation and backup flow for destructive maintenance.
- `desktop/lib/local-server.cjs`: candidate binary resolution and process
  lifecycle style reused by desktop dreaming.
- `tests/unit/memory_maintenance.rs`: existing issue #196 maintenance tests
  extended with issue #540 dreaming policy coverage.

