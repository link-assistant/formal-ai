# Issue 686 Case Study

> **Status:** Full paper-pipeline transfer, durable runtime integration, auto-learning, and Agent CLI execution implemented in PR #689.
> **Type:** Research + design + self-hosted implementation case study.
> **Primary sources:** issue body plus the maintainer's expanded acceptance criteria in [PR comment 4973111296](https://github.com/link-assistant/formal-ai/pull/689#issuecomment-4973111296).

- **Issue:** <https://github.com/link-assistant/formal-ai/issues/686>
- **Pull request:** <https://github.com/link-assistant/formal-ai/pull/689>
- **Concept → stack mapping:** [`persistence-mapping.md`](persistence-mapping.md)
- **Per-requirement plans + prior art:** [`solution-plans.md`](solution-plans.md)
- **Requirement list:** [`requirements.md`](requirements.md)
- **Online research (cited):** [`raw-data/online-research.md`](raw-data/online-research.md)
- **Reproducible Agent CLI session:** [`agent-cli-session-associative-learning.json`](agent-cli-session-associative-learning.json)
- **Real external Agent CLI E2E:** [`agent-cli-external-e2e.lino`](agent-cli-external-e2e.lino)

All raw artifacts referenced below live in [`raw-data/`](raw-data/).

## 0. Deep implementation revision (authoritative)

The original draft treated Wikontic as only a deduplication-and-degree lesson and
built an isolated `AssociativeMemory`. That was insufficient: the paper describes
a pipeline, and the store did not affect persisted application memory, dreaming,
or Agent CLI task execution. The maintainer explicitly requested deeper analysis,
automatic learning, Formal AI execution through Agent CLI, and generalization of
touched architecture. This revision closes those gaps:

1. **Qualifier-preserving candidates.** Durable `MemoryEvent`s become expression
   candidates; kind, role, intent, tool, time, and conversation context are kept
   as qualifiers rather than flattened away.
2. **Alignment and interpretable failure.** Evidence relations resolve against
   persisted ids. Unresolved relations do not disappear: the candidate remains
   with a `validation_issue`, matching the paper's retain-for-revision verifier.
3. **Normalization, deduplication, incremental writes.** Stable ids merge repeated
   assertions, namespaced evidence aliases resolve to canonical ids, and a stable-id
   rewrite now replaces stale text while incrementing the durable write counter.
4. **Links-network storage and multi-hop retrieval.** Associations remain
   `SubstitutionLink` doublets. `recall_related` performs deterministic bounded
   traversal in both directions and records each recalled expression as read.
5. **Real runtime retention.** `MemoryEvent::write_count` round-trips as
   `writeCount`, survives sync, increments on native/browser substitutions, and
   joins reads plus incoming/outgoing links in the actual dreaming eviction score.
6. **Automatic learning and Agent CLI.** The existing default-on dreaming loop now
   consumes this policy. A derived `associative-learning-report.lino` task runs
   through the generalized Agent CLI write → verify → final recipe; its report is
   computed from persisted memory, not canned prose.

The paper itself links normalization/connectivity to retrieval quality; it does
not prescribe degree-weighted eviction. Combining degree with reads/writes is the
issue author's explicit extension and is identified as an inference throughout
this case study.

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

The associative stack supplied useful substrate—`SubstitutionGraph`, `stable_id`,
and an LFU precursor—but the complete feature required six paper-pipeline
practices plus durable runtime and Agent CLI integration. PR #689 now connects the
policy to native/browser memory, sync, automatic dreaming, and the generalized
agentic document recipe. The authoritative mapping is **13 done, 2 realized
substrate** across 15 concept rows (see
[`persistence-mapping.md`](persistence-mapping.md)).

The deliverables are code + documentation, fully traceable:

1. **The implementation:**
   [`src/associative_persistence.rs`](../../../src/associative_persistence.rs) —
   `AssociativeMemory` (expressions as content-addressed nodes in a
   `SubstitutionGraph`), qualifier- and warning-bearing `PersistedExpression`,
   durable-event ingest, bounded multi-hop recall, and four-signal retention;
   `memory`, `memory_sync`, `dreaming`, and the browser mirror connect that view
   to production persistence. Executable coverage lives in
   [`tests/unit/issue_686_associative_persistence.rs`](../../../tests/unit/issue_686_associative_persistence.rs).
2. This case study, the requirement list, the concept→stack mapping, and the
   solution plans, all under `docs/case-studies/issue-686/`.
3. `REQUIREMENTS.md` rows **R445–R458** under *"Issue #686 Associative Knowledge
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
| [`raw-data/pr-689-conversation-comments.json`](raw-data/pr-689-conversation-comments.json) | Complete PR conversation, including the deeper-analysis/auto-learning/Agent-CLI requirement. |
| [`raw-data/pr-689-review-comments.json`](raw-data/pr-689-review-comments.json) | PR inline review comments — empty (`[]`). |
| [`raw-data/pr-689-reviews.json`](raw-data/pr-689-reviews.json) | PR reviews — empty (`[]`). |
| [`raw-data/online-research.md`](raw-data/online-research.md) | Summarized-and-cited research: the cited *Wikontic* paper, AriGraph, LFU/LRU cache replacement, reference counting, degree centrality, and the associative-memory model. |

Per [NON-GOALS.md](../../../NON-GOALS.md) (*"Research notes should not copy large
external texts; they should summarize and cite sources"*), the research file
quotes only short definitional phrases and links every claim to its source.

---

## 3. Holistic Requirements

Every requirement extracted from the issue body is enumerated in
[`requirements.md`](requirements.md) as **R686-01 … R686-18**. These are recorded in
[`REQUIREMENTS.md`](../../../REQUIREMENTS.md) as **R445–R458** under *"Issue #686
Associative Knowledge Networks Learning"*. The short form:

| ID | Requirement (verbatim intent) | Status |
|---|---|---|
| **R445** | Collect the issue-686 data into a dedicated case-study directory. | Done — [`raw-data/`](raw-data/). |
| **R446** | Deep case-study analysis with cited online research. | Done — this file + [`raw-data/online-research.md`](raw-data/online-research.md). |
| **R447** | Enumerate every requirement of the issue and maintainer feedback. | Done — [`requirements.md`](requirements.md) (R686-01 … R686-18). |
| **R448** | Map each persistence concept to the associative stack with honest status. | Done — [`persistence-mapping.md`](persistence-mapping.md) (13 done / 2 realized substrate). |
| **R449** | Propose a solution plan per requirement, surveying existing components/libraries. | Done — [`solution-plans.md`](solution-plans.md). |
| **R450** | Implement the usage-weighted associative persistence store. | Done — [`src/associative_persistence.rs`](../../../src/associative_persistence.rs) + [`tests/unit/issue_686_associative_persistence.rs`](../../../tests/unit/issue_686_associative_persistence.rs). |
| **R451** | Plan and execute everything in the single PR #689. | Done — every artifact here plus the changelog fragment and traceability test. |
| **R452** | Protect the case study with a documentation-traceability regression test. | Done — `tests/unit/docs_requirements_issue_686.rs`. |

See [`requirements.md`](requirements.md) for the complete R686-01 … R686-18
decomposition of the issue body and expanded maintainer acceptance criteria.

---

## 4. Deep Analysis — usage-weighted persistence on the associative stack

### 4.1 Transfer the paper's pipeline, not only its degree metric

Wikontic's transferable value is its complete structured pipeline: preserve
qualifiers while extracting candidates; refine relations against ontology/type
constraints; normalize aliases and deduplicate; extend storage incrementally;
retain verifier warnings; and answer through iterative multi-hop retrieval. The
native symbolic equivalents are now implemented over `MemoryEvent` candidates and
`AssociativeMemory`. The reported average-degree result motivates connectivity for
retrieval, while issue #686—not the paper—adds degree to the retention formula.

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
connection for each gap. The revised implementation reuses `SubstitutionGraph`,
`stable_id`, durable `MemoryEvent`s, the dreaming runtime, browser memory, and the
general Agent CLI document recipe. Determinism is honored throughout: retention is decided by usage
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
[`persistence-mapping.md`](persistence-mapping.md). Summary — **13 done, 2
realized substrate** across 15 concept
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
full **prior-art survey**. The runtime architecture — now implemented across
[`src/associative_persistence.rs`](../../../src/associative_persistence.rs),
`memory`, `memory_sync`, `dreaming`, the browser mirror, and `agentic_coding` — is a
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
  embedding memory (FAISS, HNSW, RAG) do not satisfy this issue's requirement for
  deterministic, inspectable links-network persistence.

**Net conclusion:** for every requirement, either an in-repo component already
realizes it (and is cited) or the new module supplies the exact missing connection
— no requirement is left both unrealized and unplanned.

---

## 8. Risks

| Risk | Why it matters here | Mitigation in the plan |
|---|---|---|
| **Non-determinism** | Any clock/recency signal would break byte-reproducibility (`NON-GOALS.md`). | Retention is frequency + degree only; no clocks, no randomness; ranking and `links_notation` are sorted and reproducible. |
| **Counter overflow** | A pathological read/write/degree count could wrap the score. | Every term in `retention_score_with` uses saturating arithmetic; counters saturate on increment. |
| **Silent second store** | A parallel expression store could drift from the append-only memory log. | `AssociativeMemory` is a *policy view* keyed by content-addressed ids and is rebuilt by `from_memory_events`; `from_context` remains an explicit world-model adapter. |
| **Losing links on forget** | Evicting an expression could leave dangling associations. | `forget` removes the record **and** every incident link; a regression test asserts degree drops to zero on neighbors. |
| **Overclaiming completeness** | Marketing a half-built policy would violate `NON-GOALS.md`. | Honest Done/Realized status per row; the mapping states plainly which signals were newly added vs. inherited. |

---

## 9. Files

```
docs/case-studies/issue-686/
├── README.md                 # this analysis
├── agent-cli-session-associative-learning.json # byte-reproducible in-repo driver
├── agent-cli-external-e2e.lino # real @link-assistant/agent ↔ release server
├── requirements.md           # R686-01 … R686-18 (R447)
├── persistence-mapping.md    # concept → associative-stack mapping (R448)
├── solution-plans.md         # per-requirement plans + prior-art survey (R449)
└── raw-data/                 # third-party captures (lint-exempt)
    ├── issue-686.json
    ├── issue-686-comments.json          # current issue conversation
    ├── pr-689.json
    ├── pr-689-conversation-comments.json # includes maintainer feedback
    ├── pr-689-review-comments.json       # empty
    ├── pr-689-reviews.json               # empty
    └── online-research.md               # summarized + cited (R446)
```

Implemented and wired into the rest of the repository by:

- `src/associative_persistence.rs` — the persistence policy view: `AssociativeMemory`,
  `PersistedExpression`, `RetentionWeights`, `ScoredExpression`, and the persist /
  count / degree / `retention_score` / evict / durable-event ingest / multi-hop API.
- `src/memory.rs`, `src/memory_sync.rs`, `src/dreaming.rs`, and
  `src/web/memory.js` — portable write accounting and automatic retention.
- `src/agentic_coding/associative_learning.rs` — derived-memory execution through
  the generalized Agent CLI document recipe.
- `src/lib.rs` — `pub mod associative_persistence;` plus re-exports of the public
  types.
- `tests/unit/issue_686_associative_persistence.rs` and
  `tests/unit/issue_686_agent_cli.rs` — executable coverage of persistence,
  mutation, runtime retention, multi-hop recall, and Agent CLI behavior.
- `REQUIREMENTS.md` — rows **R445–R458** (R447).
- `README.md` and `ARCHITECTURE.md` — a discoverable reference to this case study.
- `tests/unit/docs_requirements_issue_686.rs` — pins the reference, the headings,
  and the requirement IDs so the case study cannot silently regress (R452).
- `changelog.d/` — a `minor` fragment recording the feature.
