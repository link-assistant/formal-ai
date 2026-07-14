# Issue 686 — Online Research (summarized and cited)

Per [NON-GOALS.md](../../../../NON-GOALS.md) — *"Research notes should not copy
large external texts; they should summarize and cite sources"* — this file quotes
only short definitional phrases and links every claim to its source. It backs the
analysis in [`README.md`](../README.md).

## 1. The cited paper — "Wikontic" (arXiv 2512.00590)

The issue links <https://huggingface.co/papers/2512.00590>. That paper is
**"Constructing Wikidata-Aligned, Ontology-Aware Knowledge Graphs with Large
Language Models"** by Alla Chepurova, Aydar Bulatov, Yuri Kuratov, and Mikhail
Burtsev, which introduces a system named **Wikontic**. Its relevant findings:

- It **constructs a knowledge graph from open-domain text**, extracting candidate
  triplets with qualifiers, enforcing Wikidata-based type/relation constraints,
  and **normalizing entities to reduce duplication** (alias-aware matching so
  "NYC" and "New York City" resolve to one canonical node). Source: paper abstract
  via <https://huggingface.co/papers/2512.00590> and <https://arxiv.org/abs/2512.00590>.
- The resulting graphs are **"compact, ontology-consistent, and well-connected"**;
  it reports an **average entity degree of ~4.3** and the correct answer entity
  present in **96% of triplets** on MuSiQue, which it ties directly to **efficient
  multi-hop retrieval** (76.0 F1 on HotpotQA, 59.8 F1 on MuSiQue using only
  structured triplets, no raw text). Source: same.
- It is **~3× more token-efficient than AriGraph** and much cheaper than GraphRAG.
  Source: same.

**Why this matters for issue #686.** The paper's transferable "best practices" are
not its LLM extraction pipeline (out of scope — `formal-ai` is symbolic, not an
LLM KG builder) but its two structural lessons: (a) **deduplicate/normalize
entities so one meaning is one node** — which the associative stack already does
with content-addressed `stable_id`; and (b) **node degree is the currency of
retrieval efficiency** — a well-connected node is reached more often. Issue #686
lifts exactly that principle into a *retention* policy: **"calculate usages based
on incoming and outgoing links"**, i.e. let degree, not just explicit reads, drive
how long knowledge persists.

## 2. Same lab — AriGraph (the memory-graph lineage)

Kuratov and Burtsev also authored **AriGraph** (*"AriGraph: Learning Knowledge
Graph World Models with Episodic Memory for LLM Agents"*, arXiv 2407.04363), which
builds a **memory graph combining semantic and episodic knowledge** and shows that
a **structured, well-connected memory graph** outperforms flat retrieval for agent
planning. Source: <https://arxiv.org/abs/2407.04363>. It is the direct predecessor
to Wikontic and the reason the issue frames memory as a **links network world
model** rather than a vector store. This connects issue #686 to the project's
existing world-model work ([issue #649](../../issue-649/README.md)).

## 3. Usage-weighted retention — the caching / GC literature

The issue's core mechanic — *"the data that is most frequently used or changed
should persist for longer"* — is the classic **cache-eviction** problem:

- **LFU (Least-Frequently-Used)** evicts the item with the lowest access count;
  **LRU (Least-Recently-Used)** evicts the least-recently-touched. Because the
  issue mandates **determinism** (no clocks), this project uses the **frequency**
  axis (LFU-style), not the recency axis. General reference:
  <https://en.wikipedia.org/wiki/Cache_replacement_policies>.
- **Reference counting** in garbage collection reclaims an object when its
  **incoming-reference count** drops to zero — the direct analogue of the issue's
  "usages based on incoming … links". Reference:
  <https://en.wikipedia.org/wiki/Reference_counting>.
- The associative stack **already** implements an LFU-style policy for memory
  events: `dreaming::usage_counts` scores each event by `access_count` plus its
  citation in-degree and evicts the lowest first (verified at
  [`src/dreaming.rs`](../../../../src/dreaming.rs)). Issue #686's contribution is to
  add the **write/change** axis and the **outgoing-link** axis the existing policy
  omits, and to apply the policy to **persisted meta-language expressions**, not
  only to memory events.

## 4. Degree as an importance signal — network science

Ranking nodes by their link count is **degree centrality**, the simplest
centrality measure in network science: a node's importance is proportional to the
number of links incident on it (in-degree + out-degree for a directed network).
Reference: <https://en.wikipedia.org/wiki/Centrality#Degree_centrality>. Wikontic's
"entity degree ↔ retrieval efficiency" result (§1) is an applied instance: the
higher a node's degree, the more retrieval paths reach it. Issue #686 reuses
degree as one of four retention signals, alongside reads, writes, and the network
degree split into its incoming and outgoing halves.

## 5. Associative memory — why "links, not graph"

The issue insists on **"a link, or link network, not graph, not edges, not
vertices"**. This is the project's standing **doublet / Links-Notation** commitment
(see [`ARCHITECTURE.md`](../../../../ARCHITECTURE.md) and the associative-links
model of <https://github.com/linksplatform>): every unit of meaning — including an
association *between* two expressions — is itself a link, so there is a single
uniform substrate rather than a two-sorted graph of vertices-plus-edges. The
implementation honors this by storing associations in the same `SubstitutionGraph`
(a set of `SubstitutionLink` doublets) the rest of the project uses, and by
serializing the entire store — expressions, read counts, write counts, and
associations — as Links Notation.

## 6. Net conclusion

The paper contributes the **principle** (dedup to one-node-per-meaning; degree
drives retrieval); the caching/GC literature contributes the **mechanism**
(frequency-weighted retention, reference-count-style incoming-link usage); network
science contributes the **measure** (degree centrality). Issue #686 composes these
into a deterministic, glass-box, links-only retention policy over persisted
meta-language expressions — and the audit finds the associative stack already
supplies the substrate, so the work is to add the two missing signals (writes,
outgoing degree) and a first-class persistence store, not to build new
infrastructure.
