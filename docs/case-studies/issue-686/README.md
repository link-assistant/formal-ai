# Issue 686 Case Study

> **Status:** Data collected, requirements enumerated, solution plans drafted, **and the core persistence feature implemented** in PR #689.
> **Type:** Research + design case study **plus implementation** — the audit found the feature was an audit-and-wire task, and PR #689 then landed the [`src/associative_persistence.rs`](../../../src/associative_persistence.rs) module that wires it up.
> **Primary source:** the issue body (three intent paragraphs + one meta paragraph); the issue has no comments.

- **Issue:** <https://github.com/link-assistant/formal-ai/issues/686>
- **Pull request:** <https://github.com/link-assistant/formal-ai/pull/689>
- **Concept → stack mapping:** [`persistence-mapping.md`](persistence-mapping.md)
- **Per-requirement plans + prior art:** [`solution-plans.md`](solution-plans.md)
- **Requirement list:** [`requirements.md`](requirements.md)
- **Online research (cited):** [`raw-data/online-research.md`](raw-data/online-research.md)

All raw artifacts referenced below live in [`raw-data/`](raw-data/).

---

## 1. Summary

Issue #686 — *"Associative knowledge networks learning (contexts, world models,
formal systems)"* — asks the project to keep a **persistent** version of
**meta-language expressions** saved in **associative links networks**, applying the
best practices of the cited paper (arXiv 2512.00590, *Wikontic*). The concrete
mechanics the issue names are: **not only operate on facts but persist them**;
**count usages (reads) and changes (writes)**; ensure the **data most frequently
used or changed persists for longer**; **calculate usages based on incoming and
outgoing links**; and **keep everything as a link / link network — not graph, not
edges, not vertices**.

The final meta paragraph is the concrete deliverable: **collect the issue data**
into `docs/case-studies/issue-686/`, do a **deep case-study analysis** with
**online research**, **list every requirement**, **propose solution plans** per
requirement while **surveying existing components/libraries**, and **do it all in
this single PR** (#689).

The central finding was that **the associative stack already provides most of the
substrate the feature needs** — a links network (`SubstitutionGraph`), content
addressing that already gives one-node-per-meaning (`stable_id`), and an LFU-style
read-count eviction policy for memory events (`dreaming::usage_counts` +
`storage_policy`). What was missing was not infrastructure but **three
connections** the issue names explicitly and the existing machinery lacks: a
**write/change** counter (the existing policy counts reads only), an
**outgoing-link** degree signal (the existing citation count is incoming-only), and
a first-class store that persists **meta-language expressions themselves** (the
existing policies operate on memory *events*). PR #689 supplied exactly those three
in the [`src/associative_persistence.rs`](../../../src/associative_persistence.rs)
module, so the honest status is now **7 done (all via the new module), 2 realized
substrate** across 9 concept rows (see
[`persistence-mapping.md`](persistence-mapping.md)).

The deliverables are code + documentation, fully traceable:

1. **The implementation:**
   [`src/associative_persistence.rs`](../../../src/associative_persistence.rs) —
   `AssociativeMemory` (expressions as content-addressed nodes in a
   `SubstitutionGraph`), `PersistedExpression` (`id`/`text`/`reads`/`writes`),
   `RetentionWeights`, `ScoredExpression`, and the persist / count / degree /
   `retention_score` / evict / `from_context` API — with executable coverage in
   [`tests/unit/issue_686_associative_persistence.rs`](../../../tests/unit/issue_686_associative_persistence.rs).
2. This case study, the requirement list, the concept→stack mapping, and the
   solution plans, all under `docs/case-studies/issue-686/`.
3. `REQUIREMENTS.md` rows **R445–R452** under *"Issue #686 Associative Knowledge
   Networks Learning"*.
4. `tests/unit/docs_requirements_issue_686.rs` pins the reference, the headings,
   and the requirement IDs so the case study cannot silently regress.

---

## 2. Collected Data

The raw, third-party captures (exempt from authored-prose lints) are archived
under [`raw-data/`](raw-data/):

| File | What it is |
|---|---|
| [`raw-data/issue-686.json`](raw-data/issue-686.json) | The issue as filed (`gh issue view 686 --json …`). |
| [`raw-data/issue-686-comments.json`](raw-data/issue-686-comments.json) | Issue comment thread — empty (`[]`); the body is the sole specification. |
| [`raw-data/pr-689.json`](raw-data/pr-689.json) | The pull request this work lands in. |
| [`raw-data/pr-689-conversation-comments.json`](raw-data/pr-689-conversation-comments.json) | PR conversation comments — empty (`[]`). |
| [`raw-data/pr-689-review-comments.json`](raw-data/pr-689-review-comments.json) | PR inline review comments — empty (`[]`). |
| [`raw-data/pr-689-reviews.json`](raw-data/pr-689-reviews.json) | PR reviews — empty (`[]`). |
| [`raw-data/online-research.md`](raw-data/online-research.md) | Summarized-and-cited research: the cited *Wikontic* paper, AriGraph, LFU/LRU cache replacement, reference counting, degree centrality, and the associative-memory model. |

Per [NON-GOALS.md](../../../NON-GOALS.md) (*"Research notes should not copy large
external texts; they should summarize and cite sources"*), the research file
quotes only short definitional phrases and links every claim to its source.

---

## 3. Holistic Requirements

Every requirement extracted from the issue body is enumerated in
[`requirements.md`](requirements.md) as **R686-01 … R686-13** (the conceptual
feature plus the meta-deliverable). These are recorded in
[`REQUIREMENTS.md`](../../../REQUIREMENTS.md) as **R445–R452** under *"Issue #686
Associative Knowledge Networks Learning"*. The short form:

| ID | Requirement (verbatim intent) | Status |
|---|---|---|
| **R445** | Collect the issue-686 data into a dedicated case-study directory. | Done — [`raw-data/`](raw-data/). |
| **R446** | Deep case-study analysis with cited online research. | Done — this file + [`raw-data/online-research.md`](raw-data/online-research.md). |
| **R447** | Enumerate every requirement of the issue. | Done — [`requirements.md`](requirements.md) (R686-01 … R686-13). |
| **R448** | Map each persistence concept to the associative stack with honest status. | Done — [`persistence-mapping.md`](persistence-mapping.md) (7 done / 2 realized substrate). |
| **R449** | Propose a solution plan per requirement, surveying existing components/libraries. | Done — [`solution-plans.md`](solution-plans.md). |
| **R450** | Implement the usage-weighted associative persistence store. | Done — [`src/associative_persistence.rs`](../../../src/associative_persistence.rs) + [`tests/unit/issue_686_associative_persistence.rs`](../../../tests/unit/issue_686_associative_persistence.rs). |
| **R451** | Plan and execute everything in the single PR #689. | Done — every artifact here plus the changelog fragment and traceability test. |
| **R452** | Protect the case study with a documentation-traceability regression test. | Done — `tests/unit/docs_requirements_issue_686.rs`. |

See [`requirements.md`](requirements.md) §"Why these thirteen" for why the issue
decomposes into exactly these requirements and no more.

---

## 4. Deep Analysis — usage-weighted persistence on the associative stack

### 4.1 The paper's transferable lesson is degree, not the LLM pipeline

The cited paper (arXiv 2512.00590) is *Wikontic* — an LLM system that builds
Wikidata-aligned knowledge graphs, deduplicating entities so surface variants
("NYC" / "New York City") collapse to one node, and reporting that the resulting
**high entity degree** (≈4.3 average) drives **efficient multi-hop retrieval**
(online-research §1). `formal-ai` is not an LLM KG builder, so the transferable
best practices are the two *structural* ones: **one meaning is one node** —
already true here via content-addressed `stable_id` — and **degree is the currency
of retrieval/importance**. Issue #686 lifts that second lesson into a *retention*
policy: a well-connected expression is used more, so it should persist longer, and
degree can therefore *stand in for* explicit usage counting ("calculate usages
based on incoming and outgoing links").

### 4.2 "Persist, count reads and writes, keep the busy data" is LFU caching

The issue's *"we not only operate with facts we persist them, and count usages
(reads) and changes (writes) … most frequently used or changes should persistent
for longer"* is the classic **cache-eviction** problem solved by **LFU** (evict the
least-frequently-used) — chosen here over **LRU** because the project's determinism
mandate forbids wall-clocks, so frequency, not recency, is the axis
(online-research §3). The associative stack already runs an LFU policy for memory
events (`dreaming::usage_counts` scores by `access_count` + citation in-degree and
evicts lowest-first), but it counts **reads only** and applies to **events, not
persisted expressions**. The gap is therefore narrow and named: add a **write**
counter and lift the policy onto a store of expressions.

### 4.3 "Usages from incoming and outgoing links" is reference counting + degree

Deriving usage from a node's links unifies two classical ideas: **reference
counting** in garbage collection (an object's incoming-reference count decides when
it can be reclaimed) and **degree centrality** in network science (a node's
importance ∝ its incident-link count). The existing citation signal in `dreaming`
is **incoming-only**; issue #686 explicitly wants **incoming *and* outgoing**, so
the new `degree = in_degree + out_degree` — with each half independently weighted —
completes the picture (online-research §3, §4).

### 4.4 "A link, not a graph" is the standing doublet commitment

The insistence on *"a link, or link network, not graph, not edges, not vertices"*
is the project's uniform-substrate stance: an association *between* two expressions
is itself a link (a `SubstitutionLink` doublet), stored in the same
`SubstitutionGraph` the rest of the project uses, so there is no separate
vertex/edge type system. The store serializes — expressions, read counts, write
counts, and associations alike — as Links Notation (online-research §5).

### 4.5 Plan first, then implement

The issue's meta paragraph asks to *"propose possible solutions and solution plans
for each requirement"* and to *"check known existing components/libraries"* — so
this case study did the audit first: name what already realizes each requirement
(`SubstitutionGraph`, `stable_id`, `dreaming`) and name the concrete missing
connection for each gap (write counter, outgoing degree, expression store). The
audit's payoff is that the implementation is a thin wiring layer, not a green-field
engine: [`src/associative_persistence.rs`](../../../src/associative_persistence.rs)
reuses `SubstitutionGraph` as the association store, `stable_id` for content
addressing, and the `dreaming` LFU pattern for eviction, adding only the
`AssociativeMemory` store, the write/degree signals, and the combined
`retention_score`. Determinism is honored throughout: retention is decided by usage
counts and link degree, never by wall-clock time, so a replay of the same reads,
writes, and associations yields byte-for-byte the same ranking.

### 4.6 What the modern literature adds

The memory-graph literature (AriGraph and, downstream, Wikontic; online-research
§1–2) converges on *a well-connected, deduplicated links network as agent memory*,
and shows degree correlates with retrieval quality. `formal-ai`'s bet is that this
memory can be a **readable, deterministic** links network rather than an embedding
store — the same wager the project makes elsewhere (issue-649 §4.5). Nothing in the
literature contradicts a symbolic, usage-weighted memory; it simply optimizes a
different axis (learned generality) than the one this project optimizes
(inspectability and determinism).

---

## 5. Concept → Associative Stack (overview)

The full mapping with `path:symbol` evidence and status is in
[`persistence-mapping.md`](persistence-mapping.md). Summary — **7 done (all via the
new `associative_persistence` module), 2 realized substrate** across 9 concept
rows:

| Issue concept | Associative-stack realization | Status |
|---|---|---|
| Links network (not graph/edges/vertices) | `AssociativeMemory` over `SubstitutionGraph` + `links_notation` | Done |
| One meaning is one node (Wikontic dedup) | `persist` → `stable_id` content addressing | Realized + Done |
| Persist meta-language expressions | `AssociativeMemory` / `PersistedExpression` | Done |
| Count usages (reads) | `note_read` / `reads` | Done |
| Count changes (writes) | `persist` / `note_write` / `writes` | Done |
| Most used / changed persists longer | `retention_score` / `eviction_order` / `retain_most_used` | Done |
| Usages from incoming + outgoing links | `in_degree` / `out_degree` / `degree` / `link_usage` | Done |
| Bridge to world models / contexts | `from_context` (issue #649 `Context`) | Done |
| Read-count LFU eviction (precursor) | `dreaming::usage_counts` + `storage_policy` | Realized (substrate) |

---

## 6. Solution Plans

The per-requirement solution plan and the existing-component reuse target for each
requirement are in [`solution-plans.md`](solution-plans.md), which also carries the
full **prior-art survey**. The one-paragraph architecture — now implemented in
[`src/associative_persistence.rs`](../../../src/associative_persistence.rs) — is a
first-class `AssociativeMemory = { associations: SubstitutionGraph, expressions:
BTreeMap<id, PersistedExpression> }`, per-expression read/write counters, degree
computed over the association graph, a single `retention_score` = `read·reads +
write·writes + incoming·in_degree + outgoing·out_degree`, and eviction that forgets
the lowest-scored first.

---

## 7. Existing Components / Prior Art Surveyed

Detailed in [`solution-plans.md`](solution-plans.md); summary:

- **In-repo (reused, not re-bought):** `SubstitutionGraph` / `SubstitutionLink`
  (the links network), `stable_id` (content addressing / dedup),
  `dreaming::usage_counts` (the LFU read-count precursor), `storage_policy`
  (consent-gated eviction), `world_model::Context` (the ingest bridge).
- **External formalisms re-expressed:** Wikontic (dedup + degree↔retrieval),
  AriGraph (memory-graph lineage), LFU/LRU cache replacement, reference counting,
  degree centrality.
- **External systems surveyed, intentionally not adopted:** vector databases /
  embedding memory (FAISS, HNSW, RAG) — out of scope by construction because the
  issue requires links networks *instead of* embeddings.

**Net conclusion:** for every requirement, either an in-repo component already
realizes it (and is cited) or the new module supplies the exact missing connection
— no requirement is left both unrealized and unplanned.

---

## 8. Risks

| Risk | Why it matters here | Mitigation in the plan |
|---|---|---|
| **Non-determinism** | Any clock/recency signal would break byte-reproducibility (`NON-GOALS.md`). | Retention is frequency + degree only; no clocks, no randomness; ranking and `links_notation` are sorted and reproducible. |
| **Counter overflow** | A pathological read/write/degree count could wrap the score. | Every term in `retention_score_with` uses saturating arithmetic; counters saturate on increment. |
| **Silent second store** | A parallel expression store could drift from the append-only memory log. | `AssociativeMemory` is a *policy view* keyed by content-addressed ids and can be rebuilt (`from_context`) rather than being an independent source of truth. |
| **Losing links on forget** | Evicting an expression could leave dangling associations. | `forget` removes the record **and** every incident link; a regression test asserts degree drops to zero on neighbors. |
| **Overclaiming completeness** | Marketing a half-built policy would violate `NON-GOALS.md`. | Honest Done/Realized status per row; the mapping states plainly which signals were newly added vs. inherited. |

---

## 9. Files

```
docs/case-studies/issue-686/
├── README.md                 # this analysis
├── requirements.md           # R686-01 … R686-13 (R447)
├── persistence-mapping.md    # concept → associative-stack mapping (R448)
├── solution-plans.md         # per-requirement plans + prior-art survey (R449)
└── raw-data/                 # third-party captures (lint-exempt)
    ├── issue-686.json
    ├── issue-686-comments.json          # empty
    ├── pr-689.json
    ├── pr-689-conversation-comments.json # empty
    ├── pr-689-review-comments.json       # empty
    ├── pr-689-reviews.json               # empty
    └── online-research.md               # summarized + cited (R446)
```

Implemented and wired into the rest of the repository by:

- `src/associative_persistence.rs` — the persistence feature: `AssociativeMemory`,
  `PersistedExpression`, `RetentionWeights`, `ScoredExpression`, and the persist /
  count / degree / `retention_score` / evict / `from_context` API.
- `src/lib.rs` — `pub mod associative_persistence;` plus re-exports of the public
  types.
- `tests/unit/issue_686_associative_persistence.rs` — executable coverage of
  read/write counting, degree, retention ranking, eviction, forgetting,
  determinism, and `Context` ingestion.
- `REQUIREMENTS.md` — rows **R445–R452** (R447).
- `README.md` and `ARCHITECTURE.md` — a discoverable reference to this case study.
- `tests/unit/docs_requirements_issue_686.rs` — pins the reference, the headings,
  and the requirement IDs so the case study cannot silently regress (R452).
- `changelog.d/` — a `minor` fragment recording the feature.
