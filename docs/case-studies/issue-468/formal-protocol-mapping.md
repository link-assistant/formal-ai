# The Nine Primitives as Links

> Companion to [`README.md`](README.md). This document discharges requirement
> **R311**: it shows, primitive by primitive, that the protocol's nine typed
> primitives reduce to plain links (doublets) — *everything is a link* — and
> points at the exact code that performs each reduction.

A **link** (doublet) is a named directed edge `source → target`. In a fully
reduced, untyped links store a link is just an ordered pair; the project keeps an
`id` on each pair so the reduction stays legible
([`Link`](../../../src/text_formalization/links.rs) — `links.rs:27`). The role of
an edge (subject, predicate, time, …) lives in the link id's suffix, *not* in a
schema, so the typed view and the link view are the same data.

Three renderings are produced from one `KnowledgeBase`, and they agree:

| Rendering | Method | Shape |
|---|---|---|
| Typed JSON (the protocol wire format) | `to_json` / `to_json_pretty` | `{doc_id, directory?, annotations}` |
| Structured Links Notation | `to_lino` (`lino.rs`) | one record per primitive |
| **Fully reduced doublet stream** | `to_links` / `to_links_lino` (`links.rs`) | a flat list of `source → target` edges |

The JSON round-trips losslessly (`KnowledgeBase::from_json` ∘ `to_json` is the
identity, pinned by test), and the doublet stream is deterministic: identical
bases produce identical link sets in identical order, so the **count** can be
pinned. For the curated Tale that count is **115**
(`KnowledgeBase::link_count`).

---

## The reduction, primitive by primitive

Each section gives the protocol fields, the link edges they reduce to, and the
function that emits them. Literal values become shared nodes
`lit:<datatype>:<value>` so equal literals collapse to one node
(`Term::node_id` — `primitives.rs:564`; `literal_node` — `links.rs:66`).

### 1. Concept — `concept_links` (`links.rs:88`)

Fields `id, label, type, attributes` →

| Edge id | source → target |
|---|---|
| `lnk:<id>:type` | `<id>` → `Concept` |
| `lnk:<id>:label` | `<id>` → `lit:string:<label>` |
| `lnk:<id>:kind` | `<id>` → `lit:string:<type>` *(only if set)* |
| `lnk:<id>:attr:<k>` | `<id>` → `lit:string:<v>` *(per attribute)* |

### 2. Entity — `entity_links` (`links.rs:116`)

Fields `id, label, canonical_forms, attributes` →

| Edge id | source → target |
|---|---|
| `lnk:<id>:type` | `<id>` → `Entity` |
| `lnk:<id>:label` | `<id>` → `lit:string:<label>` |
| `lnk:<id>:form:<i>` | `<id>` → `lit:string:<form>` *(per canonical form)* |
| `lnk:<id>:attr:<k>` | `<id>` → `lit:string:<v>` *(per attribute)* |

The canonical-forms list — the very *"surface forms / synonyms"* the maintainer
is wary of in "entities" — is nothing but a fan of `:form:<i>` edges off the same
node. There is no entity *object*, only the node and its edges.

### 3. Predicate / Relation — `predicate_links` (`links.rs:144`)

Fields `id, name, arity, semantics` →

| Edge id | source → target |
|---|---|
| `lnk:<id>:type` | `<id>` → `Predicate` |
| `lnk:<id>:name` | `<id>` → `lit:string:<name>` |
| `lnk:<id>:arity` | `<id>` → `lit:number:<arity>` |
| `lnk:<id>:semantics` | `<id>` → `lit:string:<semantics>` *(only if set)* |

### 4. Assertion — `Assertion::to_links` (`links.rs:261`)

The protocol's *irreducible* atomic block — `id, subject, predicate, object(s),
modality, time, context, confidence, provenance` — is exactly where "everything
is a link" has to earn its keep. It reduces to:

| Edge id | source → target |
|---|---|
| `lnk:<id>:type` | `<id>` → `Assertion` |
| `lnk:<id>:subject` | `<id>` → `subject.node_id()` |
| `lnk:<id>:predicate` | `<id>` → `<predicate id>` |
| `lnk:<id>:object:<i>` | `<id>` → `object.node_id()` *(per object)* |
| `lnk:<id>:time` | `<id>` → `time:<kind>:<value>` *(only if set)* |
| `lnk:<id>:context` | `<id>` → `<context id>` *(only if set)* |
| `lnk:<id>:modal` | `<id>` → `modal:<kind>` |
| `lnk:<id>:confidence` | `<id>` → `lit:number:<confidence>` |
| `lnk:<id>:provenance` | `<id>` → `prov:<doc>:<start>-<end>` *(only if set)* |

So an assertion **is** its star of outgoing edges. Two consequences matter:

- **Nesting is free.** A `Term::AssertionRef` object reduces its `:object:<i>`
  edge to *another assertion's node id* (`Term::node_id` — `primitives.rs:564`),
  so a higher-order assertion ("the old woman demanded *that the fish make her a
  ruler*") is just an edge whose target is another assertion node. No special
  case (article §12).
- **Qualifiers are edges, not nested records.** Modality, time, context,
  confidence, and provenance — the metadata RDF needs reification or RDF-star to
  attach — are here *ordinary outgoing edges* of the assertion node. That is the
  whole point of the reduction (see §7 of the README for the RDF-star parallel).

### 5. Procedure — `procedure_links` (`links.rs:170`)

Fields `id, signature, body, triggers` →

| Edge id | source → target |
|---|---|
| `lnk:<id>:type` | `<id>` → `Procedure` |
| `lnk:<id>:signature` | `<id>` → `lit:string:<signature>` |
| `lnk:<id>:body` | `<id>` → `lit:string:<body>` |
| `lnk:<id>:trigger:<i>` | `<id>` → `<trigger predicate id>` *(per trigger)* |

A trigger edge points at a **predicate node**, so the rule "predicate `pred:grant`
fires procedure `proc:escalate`" is one edge in the same graph as the facts — the
inference rules live in the link store next to the data they act on.

### 6. Context — `context_links` (`links.rs:196`)

Fields `id, label, description, properties` →

| Edge id | source → target |
|---|---|
| `lnk:<id>:type` | `<id>` → `Context` |
| `lnk:<id>:label` | `<id>` → `lit:string:<label>` *(only if set)* |
| `lnk:<id>:description` | `<id>` → `lit:string:<description>` *(only if set)* |
| `lnk:<id>:prop:<k>` | `<id>` → `lit:string:<v>` *(per property, e.g. `location`)* |

An assertion binds to a context by id (the `:context` edge above), so the
article's per-assertion `{"id": "ctx:loc", "properties": {"location": "Москва"}}`
is one node with `:prop:location` edges, referenced by one `:context` edge.

### 7. Temporal — `temporal_node` (`links.rs:71`)

The unified time value is not a record but a **node identifier** that the
assertion's `:time` edge points at:

| Variant | node id |
|---|---|
| `Instant{value}` | `time:instant:<value>` |
| `Interval{start,end}` | `time:interval:<start>..<end>` |
| `Relative{value}` | `time:relative:<value>` |

Equal instants collapse to the same node, so "two assertions at 2019" share a
time node — temporal aggregation is just shared-target lookup.

### 8. Modal — inline on the assertion (`links.rs:307`)

Modality reduces to the assertion's `:modal` edge (`<id>` → `modal:<kind>`) and
its `:confidence` edge (`<id>` → `lit:number:<confidence>`). The protocol mandates
a modality and confidence on **every** assertion, so these two edges are always
present (`Modal::default` is `assertion`/`1.0` — `primitives.rs:344`). A
*possibility* ("the fish *might* make her a ruler", confidence 0.5) differs from a
plain assertion only in the target of one edge.

### 9. Annotation — `annotation_links` (`links.rs:228`)

Fields `id, source_doc, offsets, language, tokenization` →

| Edge id | source → target |
|---|---|
| `lnk:<id>:type` | `<id>` → `Annotation` |
| `lnk:<id>:source` | `<id>` → `<source doc>` |
| `lnk:<id>:span` | `<id>` → `span:<start>-<end>` |
| `lnk:<id>:language` | `<id>` → `lit:string:<language>` *(only if set)* |
| `lnk:<id>:token:<i>` | `<id>` → `lit:string:<token>` *(per token)* |

The grounding of a fact in source text — *"this assertion came from characters
0–37 of doc-0001, in Russian"* — is, again, a node and its edges; provenance on
the assertion side (`prov:<doc>:<start>-<end>`) points into the same span space.

---

## Whole-base reduction and the count

`KnowledgeBase::to_links` (`links.rs:335`) concatenates the per-primitive
reductions in a fixed order — concepts, entities, predicates, procedures,
contexts, annotations, then assertions — into **one** ordered doublet stream that
fully reconstructs the base. `to_links_lino` (`links.rs:349`) serializes that
stream in the same Links Notation the rest of the crate emits, and `link_count`
(`links.rs:356`) is its length.

For the curated Tale (`tale_knowledge_base`) the stream is **115 links**. That
number is pinned by a regression test, so any change to a primitive's reduction —
or any drift in the curated KB — trips the test rather than passing silently.

```
$ formal-ai formalize tale --format links | head
lnk:concept:greed:type
  source "concept:greed"
  target "Concept"
lnk:concept:greed:label
  source "concept:greed"
  target "lit:string:жадность"
lnk:concept:greed:kind
  source "concept:greed"
  target "lit:string:trait"
```

## Why this settles "as is" vs "everything is a link"

The maintainer asked to implement the protocol *as is* despite disagreeing with
entities and ontologies, *because for us everything is a link*. The two are not a
compromise here: the typed primitives (`primitives.rs`) are the protocol's own
field set, serialized to the article's own JSON; the doublet stream (`links.rs`)
is the same nine primitives with every field expanded into `source → target`
edges. Neither view is privileged — they are computed from the same
`KnowledgeBase`, and the JSON view round-trips losslessly. The protocol is
honored exactly, **and** it is shown to be, underneath, nothing but links.
