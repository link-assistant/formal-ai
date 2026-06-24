# The Nine Primitives as Links

> Companion to [`README.md`](README.md). This document discharges requirement
> **R311**: it shows, primitive by primitive, that the protocol's nine primitives
> are emitted **directly as Links Notation records** — *everything is a link* —
> and points at the exact code that emits each one. There is no typed-struct
> intermediate and no separate "reduction" step: the formalizer writes Links
> Notation, the project's own meta-language, from the start.

## What a Links Notation record is, and why it is a link

The formalizer
[`formalize_text_to_links`](../../../src/agentic_coding/formalize.rs) builds the
whole knowledge base by calling one helper for every primitive:

```rust
format_lino_record(kind, &[(key, value), …])  // src/links_format.rs
```

which produces a record of the shape

```
kind
  key "value"
  key "value"
```

In Links Notation (`lino_objects_codec`) that record **is a link**: a node headed
by `kind`, whose body is an ordered set of `key "value"` associations — and each
association is itself a doublet, a directed edge from the record to a value under
the role `key`. So a `concept` record is a link whose `id` association names the
concept and whose remaining associations are its fields; the knowledge base is a
forest of such links. Nothing in the output is a Rust struct or a
protocol-specific JSON object — the maintainer's instruction (*"we can use only
formalization using meta-language, that is already in our code base"*) is met
literally: the primitives are born as links.

`format_lino_record` sanitizes every value (carriage return, newline, tab →
escapes) and delegates to `lino_objects_codec::format::format_indented_ordered`,
the same Links Notation writer the memory and seed subsystems use, so the output
is consistent with the rest of the project.

## The header record

Every knowledge base opens with one `knowledge_base` record that names the
document, declares the primitive scheme, stamps the generator, and reports the
per-primitive counts:

```
knowledge_base
  id "tale:fisherman-and-fish"
  source "Сказка о рыбаке и рыбке"
  primitive_scheme "concept entity predicate assertion procedure context temporal modal annotation"
  generator "formal-ai/agentic-coding/formalize@links-v1"
  concepts "3"
  entities "4"
  predicates "6"
  assertions "7"
  procedures "1"
  contexts "2"
  temporals "3"
  modals "3"
  annotations "7"
```

The `primitive_scheme` field is the `PRIMITIVE_KINDS` array verbatim, so the
header itself declares the nine-primitive contract the rest of the document
fulfils.

---

## The nine primitives, record by record

Each section gives the record `kind`, the fields it carries (the `key "value"`
associations), and a real record from the canonical synopsis. Every field set is
emitted by the per-kind loops in
[`formalize_text_to_links`](../../../src/agentic_coding/formalize.rs).

### 1. Concept

Fields `id, label, type, source`. Concepts come from the recognised work's
lexicon (`type "trait"`, `source "lexicon:<doc>"`) or are abstracted from a
recognised object (`type "extracted"`, `source "<doc>"`).

```
concept
  id "concept:greed"
  label "жадность"
  type "trait"
  source "lexicon:tale:fisherman-and-fish"
```

### 2. Entity

Fields `id, label, source`. Entities are the distinct subjects/objects the
lexicon recognised in the text. The set is de-duplicated and id-sorted (a
`BTreeMap`), so the same entity is one link no matter how often it is mentioned —
the canonical-form fan the maintainer is wary of in "entities" never appears;
there is only the node and its associations.

```
entity
  id "ent:old_man"
  label "старик"
  source "tale:fisherman-and-fish"
```

### 3. Predicate / Relation

Fields `id, label, source`. One link per distinct relation the lexicon matched.

```
predicate
  id "pred:catch"
  label "поймал"
  source "tale:fisherman-and-fish"
```

### 4. Assertion — the atomic block

Fields `id, subject, subject_kind, predicate, object, object_kind`, then the
optional qualifiers `time`, `modal`, `context`, `natural_language`, and always
`annotation` + `provenance`. This is where *everything is a link* earns its keep:
the protocol's atomic fact is a single link whose `subject`/`predicate`/`object`
associations point at the entity, predicate, and concept links it relates, and
whose qualifiers are ordinary associations — no reification, no nested record.

```
assertion
  id "a:0"
  subject "ent:old_man"
  subject_kind "entity"
  predicate "pred:catch"
  object "ent:golden_fish"
  object_kind "entity"
  time "temporal:в-начале-сказки"
  context "ctx:seaside"
  annotation "ann:0"
  provenance "tale:fisherman-and-fish@0:28"
```

Two things matter:

- **Grounded triples are structured.** When the closed-class lexicon recognises a
  subject-predicate-object triple, `subject`/`predicate`/`object` are links into
  the entity/predicate catalogue and `object_kind` records whether the object is
  an `entity`, a `concept`, or a bare `literal` (assertion `a:5`'s object
  `"стать владычицей морской"` is a `literal`; `a:1`'s object `concept:ransom` is
  a `concept`).
- **Ungrounded sentences stay honest.** When the lexicon does *not* recognise a
  triple, the assertion still ships — `subject "—"`, `predicate "pred:states"`,
  the raw sentence as the `object`, plus a `natural_language` association — so the
  fact is recorded with its span but no relation is invented, and no spurious
  entity/predicate catalogue link is fabricated. The recogniser never guesses.

### 5. Procedure

Fields `id, signature, description, trigger, source`. A procedure is the work's
escalation rule; its `trigger` association points at a predicate link, so the
inference rule lives in the same link store as the facts it acts on.

```
procedure
  id "proc:escalate"
  signature "escalate(wish) -> larger_wish"
  description "после исполнения желания следующее требование возрастает"
  trigger "pred:grant"
  source "lexicon:tale:fisherman-and-fish"
```

### 6. Context

Fields `id, label, description`. An assertion binds to a context by id (the
`context` association above), so a situation/validity bound is one link referenced
by many assertions.

```
context
  id "ctx:seaside"
  label "У синего моря"
  description "место действия сказки"
```

### 7. Temporal

Fields `id, expression, kind`. The unified time value is its own link; an
assertion's `time` association points at it, so two assertions at the same time
share one temporal link.

```
temporal
  id "temporal:в-начале-сказки"
  expression "в начале сказки"
  kind "relative"
```

### 8. Modal

Fields `id, kind, degree`. Modality is a link too; an assertion's `modal`
association points at it. A commitment (degree `0.95`) differs from a desire
(degree `0.9`) only in which modal link the assertion references.

```
modal
  id "modal:commitment"
  kind "commitment"
  degree "0.95"
```

### 9. Annotation

Fields `id, doc, span, text, language`. Annotations ground a fact in its source:
one link per sentence, with **character** offsets (not byte offsets, so Cyrillic
is safe) and the detected language. Annotations are produced for *every* sentence
of *any* input, so this primitive is fully general — never guessed. The
assertion's `provenance` association (`<doc>@<start>:<end>`) points into the same
span space.

```
annotation
  id "ann:0"
  doc "tale:fisherman-and-fish"
  span "0:28"
  text "Старик поймал золотую рыбку."
  language "ru"
```

---

## The whole base, and the count

`formalize_text_to_links` emits the records in a fixed order — header, concepts
(id-sorted), entities, predicates, procedures, contexts, temporals, modals,
annotations, then assertions — into one Links Notation document. The emission is
deterministic: identical input produces an identical document, so the **record
count** can be pinned. For the canonical synopsis it is **37 records** (1 header +
3 concepts + 4 entities + 6 predicates + 1 procedure + 2 contexts + 3 temporals +
3 modals + 7 annotations + 7 assertions), and `summary.covers_all_nine()` is
`true`. Both are pinned by `tests/unit/agentic_coding.rs`, so any drift in a
primitive's fields or in the curated synopsis trips a test rather than passing
silently.

```
$ cargo run --example issue_468_formalize_text | tail -3
covered: concept, entity, predicate, assertion, procedure, context, temporal, modal, annotation
covers all nine: true
total records: 37
```

The same document is what the agentic loop's final `run_command` verification step
checks, and what an agentic CLI receives as the final assistant message — the
planner reports it as "37 records realising all nine protocol primitives" (see
[`README.md`](README.md) §5).

## Why this settles "as is" vs "everything is a link"

The maintainer asked to implement the protocol *as is* despite disagreeing with
entities and ontologies, *because for us everything is a link*. There is no
tension to resolve and no compromise: the nine primitives are present with their
protocol fields, **and** each one is literally a Links Notation record — a link
whose associations are doublets. The protocol is honored exactly, and it is
honored *in the project's own meta-language*, with no typed-struct or
protocol-specific JSON layer in between. That is a stronger reading of "everything
is a link" than a bolt-on reduction would give: the primitives never exist as
anything but links.
