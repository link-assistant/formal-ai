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

- Its pipeline has six relevant stages: a curated Wikidata ontology; candidate
  relation extraction that preserves contextual qualifiers; ontology-aware type,
  domain, and range refinement; canonical-label/alias normalization and
  deduplication; incremental structured storage; and iterative multi-hop
  retrieval. A final verifier flags ontology-misaligned relations but retains
  them for inspection and revision. Sources: the official
  [abstract](https://arxiv.org/abs/2512.00590) and
  [full HTML paper](https://arxiv.org/html/2512.00590).
- The resulting graphs are **"compact, ontology-consistent, and well-connected"**;
  it reports an **average entity degree of ~4.3** and the correct answer entity
  present in **96% of triplets** on MuSiQue, which it ties directly to **efficient
  multi-hop retrieval** (76.0 F1 on HotpotQA, 59.8 F1 on MuSiQue using only
  structured triplets, no raw text). Source: same.
- It is **~3× more token-efficient than AriGraph** and much cheaper than GraphRAG.
  Source: same.

**Why this matters for issue #686.** The first PR draft incorrectly reduced the
paper to deduplication plus degree and declared the remaining pipeline out of
scope. The maintainer's follow-up requires the ambitious generalized vision, so
the implementation transfers every symbolic practice: `MemoryEvent`s are
candidates; event metadata becomes precision-bearing qualifiers; evidence
relations are alignment-checked; namespaced references normalize to canonical
ids; stable ids deduplicate incremental ingestion; unresolved relations remain
visible as warnings; and `recall_related` performs bounded multi-hop traversal.

The paper reports connectivity and retrieval results, but it does **not** propose
degree-weighted cache retention. Treating incoming/outgoing degree as a retention
signal is an explicit inference and requirement from issue #686, combined here
with durable reads and writes; it must not be attributed to the paper itself.

## 2. Same lab — AriGraph (the memory-links lineage)

Kuratov and Burtsev also authored **AriGraph** (*"AriGraph: Learning Knowledge
Graph World Models with Episodic Memory for LLM Agents"*, arXiv 2407.04363), which
builds a **memory links network combining semantic and episodic knowledge** and shows that
a **structured, well-connected memory links network** outperforms flat retrieval for agent
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
  axis (LFU-style), not the recency axis. The USENIX replacement-policy survey
  gives these operational definitions, and the LFU-Lite analysis proves
  frequency counting can achieve order-optimal regret:
  <https://www.usenix.org/legacy/publications/library/proceedings/usits97/full_papers/cao/cao_html/node4.html>,
  <https://arxiv.org/abs/2004.00472>.
- **Reference counting** in garbage collection reclaims storage according to
  live references — the direct analogue of the issue's "usages based on incoming
  … links". A primary implementation study is Mancini and Shrivastava's
  fault-tolerant reference-count collector:
  <https://doi.org/10.1093/comjnl/34.6.503>.
- The associative stack's precursor used `access_count` plus citation in-degree.
  PR #689 now persists `writeCount` and routes live dreaming retention through
  `AssociativeMemory::from_memory_events`, which combines reads, writes, incoming
  references, and outgoing evidence links in the same policy used for eviction.

## 4. Degree as an importance signal — network science

Ranking nodes by their link count is **degree centrality**, the simplest
centrality measure in network science: a node's importance is proportional to the
number of links incident on it (in-degree + out-degree for a directed network).
Reference: Freeman's foundational centrality paper,
<https://doi.org/10.1016/0378-8733(78)90021-7>. Wikontic's
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

The paper contributes a full **symbolic pipeline** (qualifiers, ontology-aware
validation, normalization/deduplication, incremental storage, verification, and
multi-hop retrieval); the caching/GC literature contributes the **mechanism**
(frequency-weighted retention, reference-count-style incoming-link usage); network
science contributes the **measure** (degree centrality). Issue #686 composes these
into a deterministic, glass-box, links-only retention policy over persisted
meta-language expressions. The existing associative stack supplies the substrate,
but realizing the vision requires a durable write signal, both link directions,
paper-pipeline adapters, automatic-dreaming integration, portable browser/sync
formats, and self-hosted Agent CLI execution—not merely a standalone store.
