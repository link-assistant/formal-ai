# Usage-weighted persistence, expressed in the associative stack

This document maps **each concept issue #686 names** to the component in
`formal-ai` that realizes it, with `path:symbol` evidence and an honest
**Realized / Partial / Done** status. It is the issue-686 analog of
[`issue-649/world-model-mapping.md`](../issue-649/world-model-mapping.md): the
point is to show the feature is an **audit-and-wire** task over existing
associative machinery, and PR #689 then **did the wiring**, landing
[`src/associative_persistence.rs`](../../../src/associative_persistence.rs)
(`AssociativeMemory`, `PersistedExpression`, `RetentionWeights`,
`ScoredExpression`) that realizes the core of the feature.

Summary: **13 done, 2 realized substrate** across 15 concept rows. The initial audit
identified the links substrate, but maintainer feedback required the policy to run
through durable memory, dreaming, browser interoperability, and Agent CLI. The
associative stack **already** had two of the three ingredients — a links network
(`SubstitutionGraph`) and an LFU-style read-count eviction policy
(`dreaming::usage_counts`) — but was missing the **write/change** counter, the
**outgoing-link** degree signal, and a store that persists **meta-language
expressions themselves**. The new module supplies exactly those three, and ships
with executable coverage in
[`tests/unit/issue_686_associative_persistence.rs`](../../../tests/unit/issue_686_associative_persistence.rs).

| # | Issue concept | Associative-stack realization | Evidence (`path:symbol`) | Status |
|---|---|---|---|---|
| 1 | Meaning representation = **links network** (not graph/edges/vertices) | Associations are `SubstitutionLink` doublets inside a `SubstitutionGraph`; the whole store serializes as Links Notation | `src/associative_persistence.rs::AssociativeMemory::{associate, links_notation}`; substrate `src/substitution.rs::{SubstitutionGraph, SubstitutionLink}` | Done |
| 2 | **One meaning is one node** (Wikontic dedup/normalization) | Content-addressed ids — the same expression text always maps to the same node | `src/associative_persistence.rs::persist` → `src/engine.rs::stable_id` | Realized (substrate) + Done |
| 3 | **Persist** meta-language expressions (retain, not only operate) | A keyed store of `PersistedExpression` records surviving across operations | `src/associative_persistence.rs::{AssociativeMemory, PersistedExpression, persist, persist_identified}` | Done |
| 4 | **Count usages (reads)** | A per-expression read counter, incremented on recall | `src/associative_persistence.rs::{note_read, reads}` (starts at 0) | Done |
| 5 | **Count changes (writes)** | A per-expression write counter, incremented on assert/re-assert | `src/associative_persistence.rs::{persist, note_write, writes}` (asserting = 1 write) | Done |
| 6 | **Most used / most changed persists longer** (retention & eviction) | A single retention score = reads + writes + degree; eviction forgets the lowest first | `src/associative_persistence.rs::{retention_score, retention_ranking, eviction_order, evict_least_used, retain_most_used}`; prior art `src/dreaming.rs::usage_counts` (LFU on memory events) | Done |
| 7 | **Usages from incoming and outgoing links** (degree) | In-degree, out-degree, and their sum, each an independent retention signal | `src/associative_persistence.rs::{in_degree, out_degree, degree, link_usage, RetentionWeights}` | Done |
| 8 | Bridge to **world models / formal systems / contexts** | A world-model `Context`'s statements + dependency edges ingest into the store, preserving statement ids | `src/associative_persistence.rs::from_context`; reuses `src/world_model.rs::{Context, Statement, Dependency}` (issue #649) | Done |
| 9 | Read-count eviction already present (the precursor being extended) | LFU policy over memory events by `access_count` + citation in-degree | `src/dreaming.rs::usage_counts`; `src/storage_policy.rs` (consent-gated auto-free) | Realized (substrate) |
| 10 | Preserve extraction **qualifiers** | Event kind/role/intent/tool/time/conversation metadata remains attached to the persisted expression | `AssociativeMemory::from_memory_events`; `PersistedExpression::qualifiers` | Done |
| 11 | Ontology/relation **alignment with retained warnings** | Evidence endpoints are validated; unresolved relations remain inspectable | `PersistedExpression::validation_issues` | Done |
| 12 | Alias normalization, dedup, incremental extension | Stable ids merge expressions; namespaced evidence suffixes resolve to canonical ids; stable-id mutation updates text | `from_memory_events`; `persist_identified` | Done |
| 13 | Iterative **multi-hop retrieval** | Bounded deterministic traversal follows incoming and outgoing links and counts reads | `AssociativeMemory::recall_related` | Done |
| 14 | Durable runtime and automatic retention | `writeCount` survives serialization/sync/substitution; dreaming scores through the associative adapter; browser mirrors counters | `src/memory.rs`; `src/memory_sync.rs`; `src/dreaming.rs::usage_counts`; `src/web/memory.js` | Done |
| 15 | Same task through Formal AI via Agent CLI | Derived persisted-memory report follows generalized write → verify → final recipe | `src/agentic_coding/associative_learning.rs`; `tests/unit/issue_686_agent_cli.rs` | Done |

## What PR #689 implemented (the complete runtime connection)

The first audit correctly identified three policy gaps, but incorrectly treated
the standalone store as sufficient. The revised implementation closes those gaps
and connects the policy to durable memory, dreaming, browser interchange,
multi-hop retrieval, and Agent CLI execution:

- **Write/change counting (row 5).** The existing `dreaming::usage_counts` scores a
  memory event by reads (`access_count`) plus citation in-degree only — a change to
  a fact is invisible to it. `AssociativeMemory` adds a first-class `writes`
  counter so *"most frequently … changed"* is protective, exactly as the issue
  frames it. Asserting an expression is one write; re-asserting the same text is
  another (a change), so knowledge that is edited often is retained longer.
- **Outgoing-link degree (row 7).** The existing citation count is incoming-only.
  `AssociativeMemory` computes **both** `in_degree` and `out_degree` and folds them
  into the score under independent `RetentionWeights`, realizing *"calculate usages
  based on incoming **and outgoing** links"*.
- **A persistence view for expressions (rows 3–6).** The existing policies operate
  on memory *events*; issue #686 asks to persist the **meta-language expressions**
  themselves. `AssociativeMemory` is that store — content-addressed nodes,
  per-node read/write counters, an embedded association network, a combined
  retention score, and eviction that keeps the most used, most changed, and most
  connected expressions last. `from_memory_events` rebuilds that view from the
  durable event log, so it cannot become a competing source of truth.
- **The complete symbolic pipeline (rows 10–13).** Qualifiers survive extraction,
  evidence endpoints are validated without deleting uncertain claims, aliases
  resolve to canonical ids, stable-id writes replace stale text, and bounded
  bidirectional traversal supplies deterministic multi-hop recall.
- **Production execution (rows 14–15).** `writeCount` is portable across native
  serialization, sync, and the browser mirror; automatic dreaming uses the
  combined score; the same derived-memory task executes through Formal AI via
  Agent CLI with write and verification steps.

Everything is deterministic: retention is decided by usage counts and link degree,
never by wall-clock time, so replaying the same reads, writes, and associations
yields byte-for-byte the same ranking and the same Links-Notation serialization.
The [`solution-plans.md`](solution-plans.md) gives the per-requirement plan and the
external prior art each design mirrors (Wikontic degree↔retrieval, LFU/reference
counting, degree centrality — see [`raw-data/online-research.md`](raw-data/online-research.md)).
