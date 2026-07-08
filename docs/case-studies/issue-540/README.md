# Issue 540 Case Study

Issue #540 asks Formal AI to "dream" in the background: keep learning from
stored experience, generalize patterns, reorganize duplicate memory, and free
space without deleting the raw experience that cannot be recovered. Issue #494
is an explicit sub-requirement for the free-space policy: cached public data and
intermediate conclusions are disposable, while learned experience and raw event
history must be retained unless the user explicitly chooses otherwise.

## 1. Collected Data

Raw GitHub issue, related issue, PR, comment, CI, code-search, and merged-PR
captures are preserved in `docs/case-studies/issue-540/raw-data/`. The prepared
PR was #645 on branch `issue-540-daaf4da2188a`; there were no issue comments,
PR conversation comments, inline review comments, or PR reviews at collection
time.

Online research is summarized in `raw-data/online-research.md`. The relevant
patterns were background compaction, routine vacuuming, cooperative idle
scheduling, and cache eviction policy.

## 2. Requirements

The implemented slice is the deterministic memory-maintenance core:

- `R396` preserves this case study and raw evidence.
- `R397` makes dreaming a default-on background planner, not a foreground
  command that users must remember to run manually.
- `R398` keeps default dreaming non-destructive; mutation requires the existing
  explicit confirmation and backup flow.
- `R399` recalculates event usage from the memory graph before deduplicating
  recomputable records.
- `R400` protects raw messages and learned/ledger experience from automatic
  eviction.
- `R401` frees only cached public-source data, deleted-thread data, or
  recomputable intermediate conclusions.
- `R402` targets 20% free space when the caller supplies capacity/free-space
  data.
- `R403` reports when bigger storage is required instead of deleting retained
  experience.
- `R404` exposes the planner through `formal-ai memory dream`.
- `R405` schedules desktop dreaming at low priority.
- `R406` documents the design and research.
- `R407` protects the behavior with automated tests.

## 3. Root Cause

Before this change, memory had explicit destructive commands for reset and
deleted-conversation purge, but it had no intermediate maintenance layer. There
was no deterministic way to distinguish raw experience from recomputable cache
records, no recalculated usage signal for duplicate cache records, and no
machine-readable plan for preserving enough free space without over-deleting.

## 4. Implemented Design

`src/dreaming.rs` introduces a pure planner over `MemoryEvent` records. It
classifies each event into retained raw experience, retained learning,
deleted-conversation data, recomputable cache, or recomputable intermediate
data. It estimates each record's size, recalculates usage counts by scanning
event text and evidence links, groups recomputable duplicates by normalized
payload, and selects only lower-usage reclaimable candidates.

The planner is default-on via `DreamingConfig::default()`, but it does not write
memory. `apply_dreaming_plan` is a separate helper used only when the CLI caller
passes `--apply --confirm`. This mirrors the existing `purge-deleted` and
`reset` safety model.

`formal-ai memory dream` prints an inspectable plan. With storage-capacity and
free-byte inputs, it computes how many bytes must be reclaimed to maintain the
default 20% free-space target after the next incoming write. If recomputable
data is insufficient, the plan reports `requires_bigger_storage` instead of
selecting raw or learned events.

`desktop/lib/dreaming.cjs` starts a plan-only scheduler in the Electron shell.
It waits before the first run, repeats at a long interval, unrefs timers and
child processes, and uses `nice -n 19` on Unix-like platforms so foreground UI
work remains preferred.

## 5. Prior Art And Existing Components

The design reuses the existing append-only memory store, full-bundle backup
format, and destructive confirmation helper in `src/main.rs`. It follows the
same safety boundary as `MemoryStore::purge_deleted_conversations` and
`MemoryStore::reset`, but adds an inspectable planning step before any physical
deletion.

External systems reinforce the same separation:

- storage engines compact duplicate/obsolete data in background jobs;
- databases vacuum incrementally instead of shrinking everything to minimum
  size;
- browser idle callbacks schedule background work around foreground latency;
- cache systems evict only data declared cacheable and use recency/frequency
  policies under pressure.

## 6. Verification

Automated tests cover the policy directly:

- `tests/unit/memory_maintenance.rs` verifies duplicate selection by
  recalculated usage, raw/learned preservation, bigger-storage reporting, and
  explicit apply behavior.
- `desktop/scripts/dreaming.test.mjs` verifies default desktop scheduling,
  plan-only CLI arguments, low-priority wrapping, and output capture.
- `tests/unit/docs_requirements_issue_540.rs` verifies that this issue's
  requirements, research, raw data, README, architecture notes, and changelog
  remain traceable.

