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
PR conversation comments, inline review comments, or PR reviews in the initial
capture. The later maintainer amendment is preserved separately in
`raw-data/pr-645-amendment-2026-07-10.md` and drove the replay, application,
storage, and runtime work described below.

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
- `R399` recalculates event usage from memory links before deduplicating
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
- `R408` recalculates which topics the user interacts with most so learning
  concentrates where the user actually spends time.
- `R409` recovers the durable requirements the user has stated on those topics so
  the user never has to repeat himself.
- `R410` generalizes each requirement into a meta-algorithm amendment and bakes
  it into memory as retained, never-forgotten learning.
- `R411` forgets the specific task/test-run records a retained amendment can
  reproduce first under pressure, keeping the generalization.
- `R412` records the dreaming meta-algorithm as grounded, machine-readable data
  pinned to the live source.
- `R413`–`R421` require amendment application, replay verification, candidate
  simulation, recurring-structure mining, real storage and consent, cached/seed
  link usage, and true core/desktop idle scheduling.
- `R422` drives the audit through Formal AI's own Agent CLI and pins its session.

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

`formal-ai memory dream` prints an inspectable plan and measures real filesystem
capacity/free bytes unless deterministic overrides are supplied. Import paths
pass the actual incoming byte count. When pressure occurs the CLI or Electron
asks whether to enable automatic free-space maintenance and persists either
choice; accepted cleanup frees only enough for the next write. Insufficient
recomputable data reports `requires_bigger_storage` and Electron displays a
larger-storage migration prompt.

`src/dreaming_runtime.rs` starts default-on core dreaming with the server and
yields while API requests are active. `desktop/lib/dreaming.cjs` also checks
real system idle time, unrefs timers/processes, uses `nice -n 19` on Unix, and
requests low OS priority on every platform where the host supports it.

The same pass learns and generalizes. Multilingual cue data lifts standing
requirements into `MetaAlgorithmAmendment` records, while language-independent
structure mining finds repeated task forms. Dreaming derives candidate tasks
from frequent topics and replays each proposed amendment against recorded
outputs. Only a passing replay can mark a
specific as covered; failures remain explicit candidates and cannot authorize
forgetting. `apply_dreaming_plan` retains amendments and patterns idempotently,
and `src/dreaming_application.rs` reads amendments during later chat/Responses
requests so learned rules actually change future answers. The dreaming
meta-algorithm is recorded as grounded data in
`data/meta/dreaming-recipe.lino`, pinned to the live source by
`tests/unit/specification/dreaming_meta_algorithm.rs`.

### Agent CLI execution and gap generalization

This amendment was also driven through the in-repo Agent CLI against Formal
AI's OpenAI-compatible agentic planner. The canonical task asks Formal AI to
inspect the grounded dreaming recipe, identify gaps, and record the reusable
generalization resolving each gap. The live run took three turns:

1. `write_file` generated `dreaming-gap-analysis.lino` in the isolated agent
   workspace;
2. `run_command` read it back for verification;
3. Formal AI returned the verified analysis as its final answer.

The complete request, advertised tools, tool arguments/results, and final answer
are preserved in
[`agent-cli-session-dreaming-audit.json`](agent-cli-session-dreaming-audit.json).
The generated artifact is committed as
[`dreaming-gap-analysis.lino`](dreaming-gap-analysis.lino). It records all seven
observed shortfalls—unused amendments, unverified coverage, absent simulation
and pattern discovery, static storage inputs, missing consent/migration UI,
desktop-only timers, and English/terminology coupling—and the generalized stage
added for each. `tests/unit/issue_540_agent_cli.rs` reruns the loop and requires
the committed session and generated analysis to match byte-for-byte.

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
  recalculated usage, raw/learned preservation, bigger-storage reporting,
  explicit apply behavior, topic-frequency recalculation, durable-requirement
  learning, amendment generalization, covered-specific forgetting under
  pressure, and idempotent amendment materialization.
- `tests/unit/specification/dreaming_meta_algorithm.rs` keeps the dreaming
  recipe grounded: the live source still defines every named function and lists
  thirteen contiguously ordered steps.
- `desktop/scripts/dreaming.test.mjs` verifies default desktop scheduling,
  plan-only CLI arguments, low-priority wrapping, and output capture.
- `tests/unit/docs_requirements_issue_540.rs` verifies that this issue's
  requirements, research, raw data, README, architecture notes, and changelog
  remain traceable.
- `tests/unit/issue_540_agent_cli.rs` drives the Formal AI Agent CLI recipe to a
  real write/read/final loop and pins its session and gap analysis byte-for-byte.
