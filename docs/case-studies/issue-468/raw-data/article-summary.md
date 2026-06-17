# Source protocol — summary and citation

**Source:** Igor Martynov, *Формальный протокол для перевода текстов в базу
знаний* ("Formal protocol for translating texts into a knowledge base"),
telegra.ph, 2026-06-10.

**Canonical URL:**
<https://telegra.ph/Formalnyj-protokol-dlya-perevoda-tekstov-v-bazu-znanij-06-10>

This file is a structured **summary** of the protocol, written for engineering
reference. It paraphrases the article and quotes only the short formal artifacts
(JSON schema example, grammar, query) needed to build a conformant
implementation. The full prose article is the author's; read it at the URL
above. The issue body (`issue-468.json`) reproduces the author's own primitive
list verbatim; this file adds the operational detail from the linked article.

## Goals the protocol states (§1, paraphrased)

- Translate natural-language text into a structure with **unambiguous,
  machine-computable semantics**.
- **Preserve provenance, confidence and a link back to the source** for every
  unit of knowledge.
- Keep the serialization **compact** (JSON) and **composable** (units combine and
  nest).
- Make the result **queryable** and support inference/aggregation over it.

## The nine primitives (§2–§3, paraphrased)

| Primitive | Role | Fields (as given) |
| --- | --- | --- |
| Concept | abstract unit of meaning (need not be a concrete object) | id, label, type, attributes |
| Entity | a concrete object or referent | id, label, canonical_forms, attributes |
| Predicate / Relation | an operation or relation between concepts/entities | id, name, arity, semantics (formula/type) |
| Assertion | a statement / fact; the atomic block of knowledge | id, subject, predicate, object(s), modality, time, context, confidence, provenance |
| Procedure | a transformation / template / inference rule, as a function | id, signature, body (description), triggers |
| Context | situation / bounds of validity (e.g. "within RF law", "interval 1990–2000") | id, properties |
| Temporal | unified time: instant / interval / relative | type, value, granularity |
| Modal | probability / modality: belief / obligation / possibility / etc. | type, confidence |
| Annotation | a link to a span of source text + offsets + language + tokenization | id, source_doc, offsets, language, tokenization |

**Core idea (§3):** every text fragment becomes a set of **Assertions** plus
supporting **Concepts / Entities / Procedures**. Assertions are the units over
which search, inference and aggregation run.

**Difference from an ontology (§3):** an ontology is a schema (types + relations).
This format is **operational** — Assertions + procedures + modality + provenance.
The ontology remains as a *reference catalogue* (Concept / Predicate with
semantics) but holds no facts of its own.

## Authoritative JSON serialization (§ example, quoted)

A single annotated assertion, as given in the article:

```json
{
  "doc_id": "doc-0001",
  "annotations": [
    {
      "id": "a1",
      "type": "Assertion",
      "subject": { "type": "Entity", "id": "ent:petrov_petr", "label": "Пётр Петров" },
      "predicate": { "type": "Predicate", "id": "pred:open", "name": "открыл" },
      "object": [ { "type": "Entity", "id": "ent:shop_001", "label": "магазин" } ],
      "time": { "type": "Instant", "value": "2019-00-00", "granularity": "year" },
      "context": { "id": "ctx:loc", "properties": { "location": "Москва" } },
      "modal": { "type": "assertion", "confidence": 0.95 },
      "provenance": { "source_doc": "doc-0001", "offsets": [0, 37], "extractor": "nlp_v1" }
    }
  ]
}
```

This example is the conformance target for our serializer/deserializer
round-trip test.

## Grammar for assertions (BNF, quoted)

```bnf
<Assertion> ::= ASSERT <ID> SUBJECT <Term> PREDICATE <Predicate>
                OBJECT <TermList> [TIME <Temporal>] [MODAL <Modal>]
                [CONF <0..1>] [CTX <Context>] [PROV <Provenance>]
<Term>      ::= Entity(id) | Concept(id) | Literal(type, value)
<TermList>  ::= <Term> | <Term> , <TermList>
```

## Worked example sentence (§ worked example)

Input sentence: **«Пётр открыл магазин в Москве в 2019 году.»**
("Pyotr opened a shop in Moscow in 2019.")

Decomposition: subject `ent:petrov_petr`, predicate `pred:open`, object
`ent:shop_001` (магазин), time = Instant 2019 (granularity year), context
location = Москва, modality = assertion with confidence 0.95, provenance =
source span offsets `[0, 37]`. This is the sentence class our constrained
deterministic extractor targets.

## Declarative query (§9, quoted shape)

```text
SELECT ?shop
WHERE  ASSERT.subject = ent:petrov_petr
  AND  predicate      = pred:open
  AND  ctx.location   = "Москва"
  AND  time.year      = 2019
```

The protocol also sketches **procedural** queries (a `Procedure` that walks
assertions). Our implementation provides the declarative conjunctive-filter form
over the assertion set.

## Nested / higher-order assertions (§12, paraphrased)

Example: «Эксперт считает, что проект, возможно, завершится к концу 2025…»
("An expert believes the project will possibly finish by the end of 2025…").

- **Assertion A** — the expert *believes* something; its **object is another
  assertion** (B). Modality = belief, confidence ≈ 0.6.
- **Assertion B** — the project finishes by end of 2025; modality = possibility;
  may be conditional on assertion C.
- **Assertion C** — the enabling condition (e.g. financing).

This shows assertions are **first-class terms**: an `Assertion` may appear as the
subject or object of another `Assertion`. Our `Term` type therefore includes an
assertion-reference variant.

## Quality metrics the protocol proposes (§10, paraphrased)

- **Precision / Recall** of extracted assertions against a gold set.
- **Consistency** — absence of contradictions in the assertion set.
- **Provenance coverage** — fraction of assertions with a source link.
- **Fidelity / reversibility** — how well the original meaning can be
  reconstructed from the knowledge base.

## NL extraction pipeline the protocol describes (§7, paraphrased)

POS tagging → dependency parse → semantic role labeling → named-entity
recognition → coreference resolution → assertion assembly. Every stage that
generalizes to arbitrary prose is a learned model; see `online-research.md` §6.
This is why general open-domain extraction is out of scope for a deterministic
engine and is scoped as future work in the case study.

## Initial working slice the protocol suggests (§11, paraphrased)

The article recommends starting from a minimal working slice: implement the
`Assertion` record with subject/predicate/object plus time, confidence and
provenance; serialize to the compact JSON above; and add declarative querying
before attempting the full NL pipeline. Our implementation follows this ordering.
