# Architecture: Memory, Public-Knowledge Cache, and Fact Queries

Part of the [architecture overview](../../ARCHITECTURE.md) (§4). Section
numbers match the overview's table of contents; paths are relative to the
repository root.

## 4. Memory: Doublet Links, .lino Backups, and the Public-Knowledge Cache

Memory has three layers:

### 4.1 Local in-process event log

Implemented today by `rust/src/event_log.rs` and `rust/src/memory.rs`. Normal writes are
content-addressed (FNV-1a 64-bit) and appended. The log is exposed through
`SymbolicAnswer::evidence_links` (short, user-visible) and
`SymbolicAnswer::links_notation` (full trace).

Issue #196 adds explicit destructive maintenance paths on top of that normal
append-only model. `MemoryStore::purge_deleted_conversations` physically
removes events for conversations that already have a `conversation_deleted`
marker, `MemoryStore::purge_conversation` removes one conversation by id, and
`MemoryStore::reset` clears the dynamic event log. Browser IndexedDB mirrors
the same split with selected cursor deletion and full object-store `clear()`.
User-facing surfaces guard these operations with an export-first path and an
irreversible confirmation, while the CLI requires `--confirm` and can write a
full-bundle `--backup` before modifying the memory file.

### 4.2 Dreaming maintenance planner

Issue #540 adds `rust/src/dreaming.rs`, a default-on, low-priority maintenance
planner over the same `MemoryEvent` projection. Dreaming reads memory and emits
an inspectable plan; it does not mutate memory unless the caller explicitly
uses `formal-ai memory dream --apply --confirm`, with the same optional
full-bundle `--backup` as reset and purge-deleted.

The planner classifies events into five `DreamingDurability` classes:

- `IrreplaceableRaw` for raw user/assistant/system experience;
- `RetainedLearning` for learning ledgers, promoted lessons, generalized
  algorithms, and baked-in meta-algorithm amendments;
- `DeletedConversation` for data already attached to a soft-deleted thread;
- `RecomputableCache` for public-source cache and fetch/tool output;
- `RecomputableIntermediate` for derived summaries and conclusions.

Only deleted conversation data, recomputable cache data, and recomputable
intermediate data are reclaimable. Duplicate cleanup is limited to the
recomputable classes and recalculates usage by scanning current event text and
evidence links before deciding which duplicate to keep. Under storage pressure,
`DreamingConfig` targets a 20% free-space reserve by default, subtracts the
next known `incoming_bytes`, and selects the lowest-use reclaimable records
first. If reclaimable records cannot satisfy the target, the plan reports
`requires_bigger_storage` instead of selecting raw or learned experience.

Dreaming also *learns from memory links and generalizes*. `event_topic` ranks
frequent topics, `requirement_statement` reads multilingual cues from
`data/meta/dreaming-cues.lino`, and `mine_patterns` derives recurring task
structures directly from records. Proposed `MetaAlgorithmAmendment` values are
replayed against discovered candidate tasks; only an exact normalized replay
may mark a specific as covered. Applied amendments are retained as structured
`meta_algorithm_amendment` events, and `rust/src/dreaming_application.rs` reads those
events on later OpenAI-compatible requests so the learned rule changes similar
future answers without being repeated. Under pressure, only replay-verified
specifics can be forgotten via `ForgetCoveredSpecific`.

`rust/src/storage_policy.rs` measures actual filesystem capacity/free bytes and the
next incoming write. Automatic removal requires a persisted `.auto-free-space`
choice; both CLI and Electron can ask, and Electron warns when larger storage is
still required. `rust/src/dreaming_runtime.rs` runs the same learning loop in the
core server, guarded by foreground activity, while Electron additionally uses
system-idle detection and lowest practical cross-platform process priority.
After that foreground guard, issue #705's `rust/src/anticipation.rs` derives
first-order transitions over formal `IntentClass` values, ranks the top three,
expands them through observed parameters plus seeded meanings and operations,
and probes every variant offline. Unknown and failed probes enter the shared
proposal-only learning frontier one-for-one. Source prelearning is fetch-consent
gated and retains URL, digest, capture time, and TTL; ordinary solving keeps
precedence, then an exact unexpired alias may answer offline. `memory_sync`
appends a `prediction_hit` link when a later actual request matches, while the
ledger truthfully reports zero until that occurs. The seventeen-stage recipe in
`data/meta/dreaming-recipe.lino` is pinned to all of these live source modules.

The Electron desktop shell starts `desktop/lib/dreaming.cjs` by default as a
plan-only background task. It waits before its first run, repeats infrequently,
unrefs timers/processes, and wraps the CLI with `nice -n 19` on Unix-like
platforms. Operators can disable that scheduler with
`FORMAL_AI_DESKTOP_DREAMING=off`.

### 4.3 Research, learning, and stable recovery

Issue #873 joins the existing unknown trace, exact source captures, promotion
gates, and agent orchestration behind `rust/src/research_learning.rs`. Online inputs
that exhaust specialized and memory routes become grounded research tasks;
offline mode remains an explicit no-network boundary. The cycle's ordered
phases are reviewable data in `data/meta/research-learning-recovery.lino`.

External observations use `SourceReceipt`: locator and content identity are
durable while a recomputable capture payload may be evicted. Identity-matching
recollection can rehydrate that payload; changed observations append a new
receipt. Learned facts, executable procedures, and amendments to the cycle
itself share a parent-linked `KnowledgeVersion` history. Candidates never
replace the active stable pointer until every gate passes, all required baseline
ids are immutable, and immutable gates are a strict majority. Rejected
candidates remain inspectable while any prior stable version stays recoverable.

All errors enter one recovery reducer. User-led mode asks only when several
options remain; full-trust mode ranks explicit prior outcomes and tradeoffs;
per-command mode returns a permission boundary. The shared default limit is one
hour. At that bound the reducer retains the current plan in an
`AwaitingContinuation` decision and resumes only through explicit continuation,
so the bound is not represented as irreversible failure.

### 4.4 Default native link-cli / doublets-web store

Native Rust builds select `LinkStoreBackend::LinkCli` by default because
Cargo's default feature set enables `doublets-native`. The library exposes
`link_store::DefaultNativeLinkStore` and `default_native_link_store()` so
embedders can construct the active native backend without checking feature
flags themselves. Compiling with `--no-default-features` keeps the explicit
`MemoryStore` / `.lino` projection fallback for small builds and recovery
tools.

The native backend embeds the `link-cli` library and mirrors each `MemoryEvent`
into its file-mapped `doublets-rs` links network using the
`Type -> SubType -> Value` reduction in `rust/src/link_store.rs`. Every mutation is
wrapped by `GenericTransactionsDecorator`; its fsynced transition-log sidecar
commits or rolls back the complete event projection and recovers interrupted
writes when the database reopens. The HTTP server owns a binary `.links`
sidecar beside its `.lino` memory file. The portable `.lino` document is
written atomically first; if a process stops between the two projections, the
next open deterministically repairs the native sidecar from that complete
source document.

Links Notation therefore remains the deterministic projection for inspection,
backup, recovery, and migration: `import_memory_links_notation` accepts both
legacy `demo_memory` files and full `formal_ai_bundle` exports, while malformed
documents are rejected before the store is mutated. Exporting the native store
writes the same stable `.lino` event log that the CLI, HTTP, Telegram, and
browser surfaces use for portability.

Browser storage remains compatible with `doublets-web`: `js/memory.js`
uses IndexedDB for the event object store, reports `doublets-web` when a
browser doublets implementation is available, and otherwise keeps the
`indexeddb-lino-mirror` fallback. The browser and native stores therefore
share Links Notation import/export semantics even though their physical
storage engines are different.

Upstream references:

- [`link-foundation/link-cli`](https://github.com/link-foundation/link-cli)
- [`linksplatform/doublets-rs`](https://github.com/linksplatform/doublets-rs)
- [`linksplatform/doublets-web`](https://github.com/linksplatform/doublets-web)

Implemented migration surface:

1. Wrap the current memory projection in a trait so the active backend is
   swappable (`link_store::LinkStore`).
2. Enable link-cli's transactional file-mapped backend by default for native
   builds through `doublets-native`.
3. Preserve `--no-default-features` as the explicit `.lino` projection
   fallback.
4. Mirror native writes to `.lino` snapshots via
   `memory::export_links_notation`.
5. Accept existing `.lino` memory files and full bundles as migration input.
6. Keep the browser IndexedDB/doublets-web mirror on the same projection
   contract.

### 4.4 Public-knowledge cache

When the local memory does not contain enough evidence to satisfy a prompt,
the solver follows the **source cache protocol** (see
`rust/tests/unit/specification/source_cache.rs`):

- check the local `source_cache` for an entry under
  `source:wikipedia:<lang>:<slug>` (or `source:wikidata:<P|Q-id>` / 
  `source:wiktionary:<lang>:<word>`).
- if absent and `offline` is false, fetch the external source and record a
  `source:` event with `fetched_at` and `sha256`.
- if `offline` is true, refuse the fetch and record a `policy:offline`
  event.

Every external fetch ages out after `cache_ttl_seconds` (default ≈ 60 days).
This is the architectural answer to "instead of GPU and neural networks, use
reasoning with internet as a public database with our local memory as
cache."

### 4.5 Fact-query reasoning pipeline (Issue #127)

Structured factual prompts — "what is the capital of France?", "столица
Германии", "भारत की राजधानी", "中国的首都" — are answered by a dedicated
reasoning pipeline that combines the seed cache with live Wikidata calls.
The pipeline is implemented in `js/worker/formal_ai_worker.js` as
`parseFactQuestion` + `tryFactQuery` and mirrored in Rust as
`rust/src/solver_handlers/benchmark_prompts.rs::try_fact_lookup` (the offline
solver uses the seed exclusively; the browser worker reaches the live API
on cache miss).

Pipeline stages:

1. **Parse.** `parseFactQuestion(prompt, normalized)` extracts a
   `(relation, subjectTerm, language, forceFresh)` tuple. The relation
   slug — `capital`, `population`, `currency`, `official_language`,
   `continent`, `area`, `head_of_state`, `head_of_government` — anchors to
   a Wikidata property (`P36`, `P1082`, `P38`, `P37`, `P30`, `P2046`,
   `P35`, `P6`). Multilingual regexes recognize the question in en/ru/hi/zh.
2. **Cache check.** A 1-week TTL in-memory store is keyed by
   `<relation>|<subjectTerm>|<language>`. `data/seed/facts.lino` entries
   that carry a `relation` field pre-warm the cache at worker startup via
   `warmFactCacheFromSeed`, so every seeded country resolves offline. The
   user can opt out of the cache with force-fresh markers in any supported
   language (`refresh`, `не из кэша`, `ताज़ा`, `刷新`, …).
3. **Wikidata resolution.** On cache miss the worker calls
   `wbsearchentities` to map the subject term to a Q-ID, then
   `wbgetentities` to read the relation's property claim and its label /
   sitelink in the user's language.
4. **Cache store.** The resolved triple `(subject_qid, value_qid, summary,
   source_url)` is written back to the cache with the original
   `fetched_at` timestamp.
5. **Trace.** Every step is appended to the event log as a `fact_query:*`
   event (`fact_query:request`, `fact_query:relation`,
   `fact_query:subject`, `fact_query:cache:check`, `fact_query:cache:hit`,
   `fact_query:cache:miss`, `fact_query:wikidata:*`, `fact_query:response`)
   so the reasoning trace can be reconstructed from memory.

The Rust offline solver follows the same shape: when a `fact_*` record in
`data/seed/facts.lino` declares a `relation`, the matcher emits the
structured `fact_query:relation`, `fact_query:subject`,
`fact_query:cache:hit:seed`, `fact_query:subject_qid`, and
`fact_query:value_qid` events alongside the legacy `fact_lookup:*` events.
That guarantees the Rust and browser stacks agree on the evidence shape
even though only the browser stack reaches Wikidata at runtime.
