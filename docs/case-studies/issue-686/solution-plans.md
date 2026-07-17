# Issue 686 — Solution Plans & Prior-Art Survey

Per **R686-11**, this file gives a concrete solution plan for each requirement in
[`requirements.md`](requirements.md) and, for each, names the **existing component
or external library** it reuses. The revised finding (see
[`README.md`](README.md) §0) is that the first audit-and-wire implementation was
necessary but insufficient: the policy must run through durable memory,
automatic dreaming, and Agent CLI, and it must transfer the paper's whole
symbolic pipeline. Every plan below reuses
`SubstitutionGraph` (the links network), `stable_id` (content addressing), and the
`dreaming` LFU precursor rather than new infrastructure.

The classical prior art each plan mirrors is documented and cited in
[`raw-data/online-research.md`](raw-data/online-research.md); the short tags below
(Wikontic, AriGraph, LFU, reference counting, degree centrality) point into that
file.

---

## Architecture in one paragraph (implemented in `src/associative_persistence.rs`)

PR #689 landed a first-class **`AssociativeMemory`** = `{ associations:
SubstitutionGraph, expressions: BTreeMap<String, PersistedExpression> }`. A
**`PersistedExpression`** is `{ id, text, reads, writes, qualifiers,
validation_issues }` keyed by a
content-addressed `stable_id` so one meaning is one node (Wikontic dedup). Reads
and writes are counted per expression; **associations** between expressions are
`SubstitutionLink` doublets in the embedded graph, giving each node an `in_degree`
and `out_degree`. A single **`retention_score`** = `read·reads + write·writes +
incoming·in_degree + outgoing·out_degree` (weights from `RetentionWeights`)
collapses all four signals into one priority; **eviction** (`eviction_order`,
`evict_least_used`, `retain_most_used`) forgets the lowest-scored first, so the
most used, most changed, and most connected expressions persist longest.
`from_memory_events` rebuilds the policy from durable events and their evidence;
`from_context` remains the world-model adapter. Everything is deterministic (no
clocks or randomness) and serializes to Links Notation (`links_notation`).

---

## Per-requirement plans

### R686-01 — Apply the paper's best practices (Wikontic)
**Plan (Done):** transfer qualifier preservation, relation/type alignment,
alias-aware normalization and deduplication, incremental storage, retained
verification warnings, and bounded multi-hop retrieval. `MemoryEvent` provides the
symbolic candidate/qualifier input; `from_memory_events` performs normalization and
validation; `recall_related` performs retrieval. Degree additionally becomes the
issue-defined retention signal.
**Reuses:** `src/engine.rs::stable_id`, `src/substitution.rs::SubstitutionGraph`.
**Prior art:** Wikontic (entity degree ↔ retrieval, alias-aware dedup);
online-research §1.

### R686-02 — Persist meta-language expressions in an associative links network
**Plan (Done):** implemented as `AssociativeMemory` — a keyed store of
`PersistedExpression` records whose associations live in an embedded
`SubstitutionGraph`, so persistence and the links network are the same object.
`persist(text)` returns the content-addressed id; `persist_identified(id, text)`
preserves an externally-assigned id. **Reuses:** `SubstitutionGraph`, `stable_id`.
**Prior art:** associative memory / doublet stores (online-research §5).

### R686-03 — Count usages (reads)
**Plan (Done):** implemented as a per-expression `reads` counter incremented by
`note_read(id)`; a fresh expression starts at 0 reads (asserting is a write, not a
read). `reads(id)` exposes the count. **Reuses:** the existing read-count idea from
`dreaming::usage_counts` (`access_count`), generalized to expressions. **Prior
art:** LFU access counting (online-research §3).

### R686-04 — Count changes (writes)
**Plan (Done):** implemented as a per-expression `writes` counter. A brand-new
`persist` starts writes at 1 (the assertion); re-persisting the same text, or
`note_write(id)`, increments it — a change is a write. `writes(id)` exposes the
count. **Reuses:** new counter; no precursor (this is the gap the existing LFU
policy left open). **Prior art:** the write/change half of usage-weighted caching;
reference-counting mutation tracking (online-research §3).

### R686-05 — Most used / most changed persists longer
**Plan (Done):** implemented as `retention_score` (reads + writes + degree),
`retention_ranking` / `retention_scores` (most-retained first, id tie-break),
`eviction_order` (least-retained first), and the eviction helpers
`evict_least_used(n)` / `retain_most_used(capacity)` that forget the lowest-scored
first. This is an LFU-style policy generalized to expressions. **Reuses:** the
LFU-eviction pattern of `src/dreaming.rs::usage_counts` +
`src/storage_policy.rs`. **Prior art:** LFU/LRU cache replacement; reference
counting (online-research §3).

### R686-06 — Usages from incoming and outgoing links
**Plan (Done):** implemented as `in_degree`, `out_degree`, `degree`, and
`link_usage` (= degree) over the association graph; both degree halves feed
`retention_score` under independent `RetentionWeights`, so a well-connected node is
retained even with few explicit reads — realizing *"calculate usages based on
incoming **and outgoing** links"*. **Reuses:** `SubstitutionGraph::links`.
**Prior art:** degree centrality; reference counting; Wikontic degree↔retrieval
(online-research §1, §4).

### R686-07 — Keep everything as a link / link network
**Plan (Done):** associations are `SubstitutionLink` doublets — no separate
edge/vertex types — and `links_notation` renders the entire store (each
expression's text, reads, writes, and every association) as Links Notation, sorted
for byte-for-byte reproducibility. **Reuses:** `SubstitutionGraph`,
`SubstitutionLink`, the project's Links-Notation convention. **Prior art:** the
doublet / associative-links model (online-research §5); the project's standing
"everything is a link" commitment.

### R686-08 … R686-11 — Paper pipeline
**Plan (Done):** adapt paper stages to symbolic events: metadata becomes
qualifiers; evidence endpoint checks become ontology alignment; stable ids and
namespaced suffix resolution provide incremental normalization/deduplication;
unresolved evidence is retained with a warning; deterministic bidirectional BFS
provides bounded multi-hop recall. **Reuses:** `MemoryEvent`, `stable_id`,
`SubstitutionGraph`; **prior art:** Wikontic stages 1–6 and final verification.

### R686-12 — Durable auto-learning integration
**Plan (Done):** add monotone `write_count` beside `access_count`, serialize it as
`writeCount`, merge it across sync, increment it only when a stored event is
changed, and rebuild the associative policy view inside `dreaming::usage_counts`.
Mirror the portable fields and substitution increment in browser IndexedDB.
**Reuses:** `MemoryStore`, `SyncStore`, `dreaming`, `storage_policy`, `memory.js`.

### R686-13 — Formal AI via Agent CLI
**Plan (Done):** parse a persisted Links Notation scenario, derive a ranked
multi-hop report through production `AssociativeMemory`, then use the generalized
`DocumentRecipe` to make any compatible Agent CLI write, verify, and report the
artifact. Pin different prompt phrasings, tool-authoring behavior, derived-output
sensitivity, and a real external CLI session. **Reuses:** issue-538 Agent CLI
harness, `run_agentic_task`, `plan_document_recipe`.

### R686-14 … R686-18 — Evidence and delivery
**Plan (Done):** refresh all raw GitHub APIs; deepen primary-source research;
maintain this per-requirement plan/mapping; reproduce stable-id mutation and
runtime integration defects with automated tests; merge current `main`; land all
work in PR #689. **Reuses:** case-study conventions and repository CI guidance.

---

## Existing Components / Prior Art Surveyed (R686-11)

What the field and the repo already built, and what the persistence store reuses
vs. re-expresses.

### The new module (PR #689)
- **`associative_persistence`** (`src/associative_persistence.rs`) — the feature
  itself: `AssociativeMemory`, `PersistedExpression`, `RetentionWeights`,
  `ScoredExpression`, and the persist / count / degree / score / evict / ingest
  API. A thin wiring layer over the components below.

### In-repo components (reused, not re-bought)
- **`SubstitutionGraph` / `SubstitutionLink`** (`src/substitution.rs`) — the links
  network with doublet CRUD. *The association store and the "everything is a link"
  substrate.*
- **`stable_id`** (`src/engine.rs`) — FNV-1a content addressing. *One meaning is
  one node (Wikontic dedup).*
- **`dreaming::usage_counts`** (`src/dreaming.rs`) — rebuilds the associative
  policy view from durable events and ranks them by reads, writes, and both link
  directions. *The former read + citation-in-degree precursor is now the production
  consumer of the generalized policy.*
- **`storage_policy`** (`src/storage_policy.rs`) — consent-gated, write-driven
  auto-free-space. *The pressure-driven eviction context this policy plugs into.*
- **`world_model`** (`src/world_model.rs`, issue #649) — `Context`, `Statement`,
  `Dependency`. *`from_context` ingests these, bridging persistence to world
  models / formal systems / contexts.*

### External formalisms / literature (re-expressed in the associative stack)
- **Wikontic** (arXiv 2512.00590; the cited paper) — LLM-built, Wikidata-aligned
  KG with alias-aware dedup and high entity degree for efficient multi-hop
  retrieval. *Re-expressed*: qualifier-preserving candidates, alignment warnings,
  alias normalization/deduplication, incremental storage, and bounded multi-hop
  recall; degree is additionally used as the issue-defined retention signal.
- **AriGraph** (arXiv 2407.04363; same lab) — episodic+semantic memory links
  network for agents. *Confirms the "well-connected links network as memory" framing that
  motivates degree-weighted retention.*
- **LFU / LRU cache replacement** — evict by lowest frequency / oldest use.
  *Re-expressed*: the frequency (LFU) axis, chosen over recency because
  determinism forbids clocks; `retention_score` is the frequency score.
- **Reference counting (GC)** — reclaim when incoming references hit zero.
  *Re-expressed*: in-degree as a usage/retention signal (`in_degree`,
  `link_usage`).
- **Degree centrality** (network science) — importance ∝ incident-link count.
  *Re-expressed*: `degree` = in + out, folded into the score.

### External systems surveyed, intentionally not adopted
- **Vector databases / embedding memory** (FAISS, HNSW, RAG stores) — retrieval by
  learned similarity. These were evaluated but not selected: this issue requires
  a links network and a glass-box, deterministic policy that similarity search
  does not provide.

**Net conclusion:** for every requirement, either an in-repo component already
realizes it (and is cited) or the new module supplies the exact missing
connection — no requirement is left both unrealized and unplanned.
