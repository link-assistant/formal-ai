# Issue 540 Solution Plans

## Plan A: Pure Planner First

Add a deterministic planner over the existing `MemoryEvent` log. It classifies
events, estimates reclaimable bytes, recalculates usage counts from evidence,
and emits an ordered action list. This is the selected plan because it gives
tests and users an inspectable result before any mutation.

Acceptance gates:

- duplicate recomputable events are compared by recalculated usage;
- retained raw/learning events are never selected;
- the 20% free-space target can be evaluated with capacity/free inputs;
- insufficient reclaimable space is reported as a storage migration need.

## Plan B: Explicit Apply

Apply the planner only through an explicit helper and CLI switch. This preserves
the repository's existing safety rule: normal memory is append-only, physical
deletion requires confirmation, and callers should make a full-memory backup.

Acceptance gates:

- `formal-ai memory dream` prints a plan by default;
- `--apply` refuses to write without `--confirm`;
- `--backup` writes the same `formal_ai_bundle` backup shape used by reset and
  purge-deleted;
- tests prove apply removes only selected event ids.

## Plan C: Core And Desktop Background Scheduler

Start dreaming with the core server and Electron main process. Both yield to
foreground activity; desktop additionally uses actual system-idle time and the
lowest practical OS process priority.

Acceptance gates:

- default desktop status includes the dreaming scheduler;
- `FORMAL_AI_DESKTOP_DREAMING=off` disables it;
- Unix-like platforms wrap the CLI with `nice -n 19` and all platforms request
  low host priority where supported;
- scheduler tests run without launching Electron.

## Plan D: Verified Learning And Application

Derive frequent-topic candidate tasks, replay proposed amendments, retain
recurring structures, and read stored amendments on later protocol requests.
Only replay-verified coverage may make a specific record reclaimable.

## Plan E: Real Storage And Persisted Consent

Measure the filesystem hosting memory, include actual incoming bytes, and ask
before automatic cleanup. Persist both acceptance and refusal. If reclaimable
links cannot meet the 20% reserve, show a larger-storage migration prompt rather
than deleting raw experience or retained learning.

## Existing Component And Library Survey

| Requirement | Existing components surveyed | Reuse decision |
| --- | --- | --- |
| Memory garbage collection and reserve | RocksDB leveled/tiered/FIFO compaction; SQLite incremental vacuum; `redb::Database::compact`; existing `MemoryStore` purge/reset | Adopt incremental, declared-cache-only policy; retain Formal AI durability/replay semantics in the planner instead of migrating storage engines. |
| Real disk capacity | Rust `fs2`; `sysinfo::Disk`; manual `statvfs`/platform commands | Use `fs2` because it directly measures total/available bytes for the filesystem containing the memory path without mount enumeration or subprocess parsing. |
| Duplicate restructuring | RocksDB compaction filters; Redis LFU/LRU eviction; existing event evidence links | Recalculate current link references and deduplicate only recomputable records; storage engines cannot infer retained-learning safety. |
| Idle and low-priority scheduling | browser `requestIdleCallback`; Electron `powerMonitor`; Unix `nice`; Node `os.setPriority`; core request activity counters | Use host idle/priority APIs plus shared core foreground guards; timers only schedule checks, not unconditional work. |
| Pattern and trend mining | existing Formal AI topic labels, task/test-run records, grounded recipe/data conventions | Mine recurring task structures from stored events and retain them as data; avoid adding a statistical/ML dependency for deterministic first-stage mining. |
| Consent and migration | existing destructive CLI confirmation; Electron native dialogs; `.lino` sidecar preferences | Reuse native surfaces and persist both choices in a shared `.auto-free-space` convention. |
| Agentic task execution | existing generalized `DocumentRecipe` and in-repo Agent CLI driver | Add a dreaming-audit recipe that writes, verifies, and returns the gap analysis; pin its complete session byte-for-byte. |

The detailed source links and tradeoffs are preserved in
`raw-data/online-research.md`.
