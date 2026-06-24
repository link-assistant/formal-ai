# Online research — text-to-knowledge-base formalization (issue #468)

This file collects external facts and prior art that back the deep analysis in
`../README.md`. It is raw collected data: it summarizes and cites sources rather
than reproducing them. Every claim links to the source it came from.

The issue asks us to turn a text (canonical input: Pushkin's *The Tale of the
Fisherman and the Fish*) into a knowledge base built from nine primitives —
Concept, Entity, Predicate/Relation, Assertion, Procedure, Context, Temporal,
Modal, Annotation — following a "formal protocol for translating texts into a
knowledge base". The protocol is assertion-centric: each text fragment becomes a
set of `Assertion` records plus supporting `Concept`/`Entity`/`Procedure`
records, and assertions are the units over which search, inference and
aggregation happen. The sections below map every design decision in that
protocol to established work, so the case study can state plainly what is novel,
what is conventional, and what is out of reach for a deterministic engine.

## 1. Sentence → semantic graph: Abstract Meaning Representation (AMR)

AMR represents the meaning of a sentence as a rooted, directed acyclic graph
whose nodes are concepts (events, entities, properties) and whose edges are
semantic relations such as predicate–argument roles. Sentences that mean the
same thing are meant to receive the same AMR even when worded differently, which
is exactly the "unambiguous, normalized meaning" goal the protocol states in its
§1. AMR builds on PropBank frames for predicate argument structure.

- [Abstract Meaning Representation — Wikipedia](https://en.wikipedia.org/wiki/Abstract_Meaning_Representation)
- [Survey of Abstract Meaning Representation: Then, Now, Future (arXiv 2505.03229)](https://arxiv.org/abs/2505.03229)
- [Bridging Natural Language and ASP: A Hybrid Approach Using LLMs and AMR Parsing (arXiv 2511.08715)](https://arxiv.org/pdf/2511.08715)

Relevance: the protocol's `Assertion(subject, predicate, object)` is a
coarse-grained, JSON-serializable cousin of an AMR predicate-argument frame. AMR
is the academic reference point for "normalize a sentence to a graph of typed
nodes and labeled edges." AMR parsing in the general case is a learned (neural)
task — which is why the protocol's §7 NL pipeline is out of reach for a purely
deterministic engine and is scoped as future work in our analysis.

## 2. Schema-free triples: Open Information Extraction (OpenIE)

OpenIE extracts `(subject, relation, object)` triples from sentences *without a
predefined schema* — the relation name is typically just the text linking the
two arguments. For "Barrack Obama became the US President in the year 2008" an
OpenIE system yields triples such as `(Barack Obama; became; US President)` and
`(Barack Obama; became US President in; 2008)`.

- [Stanford OpenIE](https://nlp.stanford.edu/software/openie.html)
- [milIE: Modular & Iterative Multilingual Open Information Extraction (arXiv 2110.08144)](https://arxiv.org/abs/2110.08144v2)
- [CompactIE: Compact Facts in Open Information Extraction (arXiv 2205.02880)](https://arxiv.org/pdf/2205.02880)

Relevance: the protocol's worked example sentence — «Пётр открыл магазин в
Москве в 2019 году.» → subject `ent:petrov_petr`, predicate `pred:open`, object
`ent:shop_001`, time 2019, location Москва — is precisely an OpenIE-style triple
*plus* slots for time, place, modality, confidence and provenance. OpenIE shows
the triple core is well-trodden; the protocol's contribution is the metadata
envelope around the triple, not the triple itself.

## 3. Statement-level metadata: RDF reification, RDF-star, named graphs

Plain RDF triples cannot carry per-statement metadata (who said it, how
certain, when valid). Three established techniques fix this:

- **RDF reification** — mint a node standing for the statement and attach
  `rdf:subject`/`rdf:predicate`/`rdf:object` plus any metadata. Simple but
  verbose; it multiplies triple count.
- **N-ary relations** — mint a node for the *relation instance* and hang each
  argument and qualifier off it.
- **RDF-star** — lets a triple be the subject of another triple, so
  statement-level annotations (confidence, weight, temporal validity,
  provenance) attach without the reification blow-up.
- **Named graphs** — group triples under an IRI that then carries provenance.

Sources:

- [What Is RDF-star — Ontotext](https://www.ontotext.com/knowledgehub/fundamentals/what-is-rdf-star/)
- [RDF-star and SPARQL-star — GraphDB documentation](https://graphdb.ontotext.com/documentation/11.3/rdf-sparql-star.html)
- [Uncertainty Management in the Construction of Knowledge Graphs: a Survey (arXiv 2405.16929)](https://arxiv.org/pdf/2405.16929)
- [\[Citation needed\]: provenance with RDF-star — metaphacts](https://blog.metaphacts.com/citation-needed-provenance-with-rdf-star)

Relevance: the protocol's `Assertion` *is* a reified statement — it has its own
`id` and carries `modality`, `time`, `context`, `confidence`, `provenance` as
first-class fields. This is the single most important external grounding: the
protocol independently reinvents reification / RDF-star. It also confirms our
reduction is sound — a reified statement is itself a node that other links point
at, which is exactly how we lower an `Assertion` to a reified link with metadata
doublets hanging off its identifier.

## 4. Operational qualifiers in a deployed KB: Wikidata

Wikidata is the largest deployed example of statements-with-metadata. Each
statement can carry **qualifiers** (e.g. temporal validity, location),
**references** (provenance) and a **rank** (preferred / normal / deprecated) used
to filter at query time. Formal treatments model a Wikidata statement as a
high-arity predicate over `entity × property × value × validity × causality ×
sequence × annotations × provenance`.

- [Help:Qualifiers — Wikidata](https://www.wikidata.org/wiki/Help:Qualifiers)
- [Help:Ranking — Wikidata](https://www.wikidata.org/wiki/Help:Ranking)
- [SPARQL/WIKIDATA Qualifiers, References and Ranks — Wikibooks](https://en.wikibooks.org/wiki/SPARQL/WIKIDATA_Qualifiers,_References_and_Ranks)
- [Handling Wikidata Qualifiers in Reasoning (arXiv 2304.03375)](https://arxiv.org/pdf/2304.03375)

Relevance: Wikidata's qualifier/reference/rank triad maps almost one-to-one onto
the protocol's `time`+`context` (qualifiers), `provenance` (references) and
`confidence`+`modality` (a richer analogue of rank). It is direct evidence that
the protocol's metadata envelope is the *normal* shape a real-world knowledge
base converges on, not an idiosyncrasy.

## 5. Predicate inventories: FrameNet and PropBank

The protocol's `Predicate` primitive (fields `id`, `name`, `arity`, `semantics`)
is the design point that FrameNet and PropBank standardize:

- **PropBank** annotates verbs with numbered arguments `Arg0…ArgN`, where `Arg0`
  ≈ proto-agent and `Arg1` ≈ proto-patient — a fixed-arity predicate signature.
- **FrameNet** groups predicates that evoke the same *frame* (stereotyped
  situation) and names roles semantically (e.g. Buyer, Goods, Seller).

Sources:

- [The Proposition Bank: An Annotated Corpus of Semantic Roles — Palmer et al. (PDF)](https://www.cs.rochester.edu/~gildea/palmer-propbank-cl.pdf)
- [Frame-Semantic Parsing — Computational Linguistics (MIT Press)](https://direct.mit.edu/coli/article/40/1/9/1461/Frame-Semantic-Parsing)

Relevance: confirms that a predicate with an explicit `arity` and a stable `id`
is the conventional way to model relations, and that a reusable predicate
inventory (our `pred:open`, `pred:catch`, `pred:ask`, `pred:grant`, …) is the
standard reference structure. It supports the protocol's claim that the ontology
(Concepts + Predicates with semantics) is a *reference catalogue* that holds no
facts itself — facts live only in Assertions.

## 6. The general NL → KB pipeline (and why it needs learning)

Surveys of automatic knowledge-graph construction describe a three-stage
pipeline: **Named Entity Recognition** (find entities), **coreference
resolution** (cluster mentions of the same entity), then **relation extraction**
(connect entities). **Semantic Role Labeling (SRL)** — "who did what to whom,
where, when" — is the standard bridge from a parsed sentence to a frame-oriented
graph.

- [A Comprehensive Survey on Automatic Knowledge Graph Construction — ACM Computing Surveys](https://dl.acm.org/doi/10.1145/3618295)
- [Knowledge Graph Construction: Extraction, Learning, and Evaluation — Applied Sciences (MDPI)](https://www.mdpi.com/2076-3417/15/7/3727)
- [Semantic Role Labeling for Knowledge Graph Extraction from Text (arXiv 1811.01409)](https://arxiv.org/abs/1811.01409)
- [Semantic Role Labeling: A Systematical Survey (arXiv 2502.08660)](https://arxiv.org/html/2502.08660v1)
- [Evaluating the Knowledge Graph Construction with LLMs — HAL](https://hal.science/hal-04862235v1/document)

Relevance: this is the protocol's §7 pipeline (POS → dependency → SRL → NER →
coreference → assertion assembly). Every stage that generalizes to arbitrary
prose is a *learned* model. formal-ai is a deterministic, symbolic engine with
no neural inference (a deliberate project NON-GOAL), so general open-domain
extraction is correctly scoped as future work. What a deterministic engine *can*
do — and what we ship — is (a) the data model and serialization for all nine
primitives, (b) a curated, citable knowledge base for the canonical Tale, and
(c) a constrained pattern extractor for the protocol's worked-example sentence
class, with the boundary stated honestly.

## 7. "Everything is a link": the associative model and double-functorial views

The maintainer's standing constraint is that *for this project everything is a
link*; entities and ontologies are accepted only because we "implement
requirements as is" even when we disagree. The associative data model is the
external grounding for that stance: it stores data as **items** (identifier +
name) and **links** (identifier + source + verb + target), where the source,
verb or target of a link may itself be another link. It is explicitly described
as close to RDF subject–predicate–object semantics.

- [The associative data model — overview](https://testbook.com/gate/associative-data-model-in-dbms-notes)
- [Representing Knowledge and Querying Data using Double-Functorial Semantics (arXiv 2403.19884)](https://arxiv.org/pdf/2403.19884)
- [Associative Knowledge Graphs and Knowledge Models — Grape Up](https://grapeup.com/blog/associative-knowledge-graphs/)

Relevance: because a link's endpoints can themselves be links, the associative
model can *reify* — a link standing for a statement is a first-class node other
links point at. That is the precise mechanism by which we lower all nine
protocol primitives to pure links/doublets (one link per identity, one doublet
per attribute/role, a reified link per Assertion with metadata doublets). The
reduction is the project's central reconciliation: implement the entity/ontology
protocol exactly as specified, then show it collapses, with no loss, into the
links-only substrate the project actually believes in.

## 8. Source text provenance — the Tale itself

*The Tale of the Fisherman and the Fish* (Russian: «Сказка о рыбаке и рыбке»)
is an 1833 verse fairy tale by Alexander Pushkin; it is in the public domain.

- [The Tale of the Fisherman and the Fish — Wikipedia](https://en.wikipedia.org/wiki/The_Tale_of_the_Fisherman_and_the_Fish)

Relevance: our curated knowledge base encodes *facts of the plot* (the old man
catches the golden fish; the fish offers a ransom; the old woman escalates her
demands; the sea darkens with each demand; the final wish returns them to the
broken trough). We do not reproduce the poem's text. This keeps the artifact a
formalization of meaning, consistent with the protocol's provenance goal and
with the project's NON-GOAL of redistributing source material.

## Summary table — protocol primitive → external grounding

| Protocol primitive / field | Established analogue | Primary source |
| --- | --- | --- |
| Assertion (subject/predicate/object) | OpenIE triple; AMR predicate-argument frame | OpenIE; AMR |
| Assertion with id + metadata | RDF reification / RDF-star / named graph | Ontotext; arXiv 2405.16929 |
| confidence / modality | uncertainty in KGs; Wikidata rank | arXiv 2405.16929; Wikidata Ranking |
| time (instant/interval/relative) | Wikidata temporal qualifiers | Wikidata Qualifiers |
| context | Wikidata qualifiers; named graphs | Wikidata; Ontotext |
| provenance | RDF-star provenance; Wikidata references | metaphacts; Wikidata |
| Predicate (id/name/arity/semantics) | PropBank numbered args; FrameNet frames | Palmer et al.; MIT Press |
| Concept vs Entity | AMR concept nodes; KG entity typing | AMR; ACM survey |
| Annotation (offsets/tokenization) | KG-construction source grounding | ACM survey; MDPI |
| §7 NL pipeline (POS/SRL/NER/coref) | automatic KG construction pipeline | ACM survey; arXiv 2502.08660 |
| reduction of all of the above to links | associative data model (items + links) | associative model; arXiv 2403.19884 |
