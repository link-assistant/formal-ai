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

## Library Survey And Selection

The amendment asked explicitly whether mature libraries could replace bespoke
storage/runtime work. The implementation keeps the Formal-AI-specific policy
(durability, replay coverage, and retained learning) local, but delegates real
filesystem statistics:

| Candidate | Existing capability | Decision |
| --- | --- | --- |
| [`fs2`](https://docs.rs/fs2/latest/fs2/) | Direct cross-platform `total_space` and `available_space` calls for the filesystem containing a path. | Selected: small dependency and exact path-oriented API needed by `.lino` memory. |
| [`sysinfo::Disk`](https://docs.rs/sysinfo/latest/sysinfo/struct.Disk.html) | Enumerates mounts and reports total/available space plus broader system/process data. | Not selected: substantially broader than the two filesystem values required, and mount matching would add policy. |
| [`redb::Database::compact`](https://docs.rs/redb/latest/redb/struct.Database.html#method.compact) | Embedded Rust database compaction. | Deferred: the active issue path is the portable Links Notation projection; changing the storage engine would not implement semantic retention or replay verification. |
| [SQLite incremental vacuum](https://www.sqlite.org/pragma.html#pragma_incremental_vacuum) | Reclaims a caller-selected number of freelist pages when incremental auto-vacuum is enabled. | Pattern adopted, engine not adopted: reclaim only enough for current pressure, but avoid a database migration unrelated to this issue. |
| [RocksDB compaction](https://github.com/facebook/rocksdb/wiki/Compaction) | Background leveled/tiered/FIFO compaction, including FIFO for cache-like data. | Pattern adopted, engine not adopted: low-priority background compaction and declared cacheability match dreaming, but RocksDB cannot infer Formal AI's retained-learning semantics. |

This survey is why `src/storage_policy.rs` uses `fs2`, while the deterministic
planner remains responsible for deciding which memory links are genuinely
recomputable and whether replay verified a generalization before deletion.
