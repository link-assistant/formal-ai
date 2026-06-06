# Meaning Data Model Alternatives for Issue #398

This note answers the PR review request for five alternative variants to the
current meaning data structure. The issue asks for meanings, words, senses,
notation, annotation, denotation, and connotation to be represented as links
instead of English-only scalar descriptions.

## Decision Criteria

- Every semantic assertion should be inspectable as meaning-to-meaning data.
- Word surfaces should be separate from lexical senses and grammatical facts.
- Source grounding should preserve references, cache records, and local
  overrides.
- Startup seed data should stay compact while importers can expand the network
  on demand.
- The model should work for formal types and natural-language meanings without
  privileging English.

## Variant 1: Recursive Facet Seed

Shape:

```lino
meaning "link"
  facet "notation"
    meaning "links_notation_format"
  facet "denotation"
    meaning "relation"

word "semantic facet"
  facet "notation"
    meaning "word_surface"
  facet "denotation"
    meaning "semantic_facet"
  facet "part_of_speech"
    meaning "noun_phrase"
```

Pros:
- Fits the existing `.lino` parser and seed files.
- Lets facet kinds be data (`notation`, `denotation`, `part_of_speech`) rather
  than Rust enum variants.
- Supports cycles and self-reference naturally.

Cons:
- Does not by itself distinguish assertion provenance, confidence, source rank,
  or local override status.
- If used alone, it can become a loose graph of unlabeled conventions.

Best use:
This is the best incremental shape for the current repository. PR #399 now uses
it for meaning-level facets and word-form facets.

## Variant 2: Reified Statement Model

Shape:

```lino
statement "semantic_facet_denotes_relation"
  subject "semantic_facet"
  predicate "denotation"
  object "relation"
  qualifier "scope"
    meaning "seed_foundation"
  reference "source"
    meaning "wikidata_data_model"
```

Pros:
- Handles provenance, qualifiers, rank, and conflicts explicitly.
- Maps well to Wikidata's statement, qualifier, and reference pattern.
- Can represent source imports and local overrides without changing the base
  meaning node.

Cons:
- More verbose than direct facets.
- Needs query helpers so common reads do not become expensive or awkward.

Best use:
Use this for imported facts, external-source comparisons, cache records, and
overrides. Direct facets can remain the compact local view; statements can be
the audit trail behind them.

## Variant 3: OntoLex-Style Lexical Entries and Senses

Shape:

```lino
lexical_entry "semantic_facet_en_noun_phrase"
  canonical_form "semantic facet"
  language "en"
  part_of_speech "noun_phrase"
  sense "semantic_facet_en_sense"

lexical_sense "semantic_facet_en_sense"
  lexical_entry "semantic_facet_en_noun_phrase"
  denotation "semantic_facet"
```

Pros:
- Separates a written form from the sense it denotes.
- Gives a clean home to part of speech, morphology, inflection, examples, and
  translation equivalents.
- Aligns with OntoLex-Lemon and Wikidata lexeme/form/sense imports.

Cons:
- Bigger migration because the current seed stores word forms inline under a
  meaning.
- Requires compatibility APIs for existing handlers that query `word_in` and
  `words_for_role`.

Best use:
Use this as the target model for word and sense backfill. PR #399 starts the
bridge by allowing inline `word` forms to carry meaning-linked facets.

## Variant 4: SKOS Concept Scheme Model

Shape:

```lino
concept_scheme "core_ontology"
  root "link"

concept "relation"
  broader "link"
  related "semantic_facet"
  pref_label "en"
    surface "relation"
```

Pros:
- Good for taxonomies, thesauri, broader/narrower/related mappings, and source
  alignment.
- Directly supports the issue's request to compare against thesauri and
  dictionaries.
- Helps classify whether a source match is exact, broader, narrower, or related.

Cons:
- Not enough by itself for lexical senses, morphology, or formal-type
  assertions.
- Labels can become language text again unless paired with lexical-entry or
  word-form records.

Best use:
Use this for concept hierarchy imports and source-comparison reports, not as the
only internal meaning model.

## Variant 5: Typed Hyperlink or Holon Model

Shape:

```lino
link "semantic_facet_denotation_relation"
  type "denotation"
  part "subject"
    meaning "semantic_facet"
  part "object"
    meaning "relation"
  whole "semantic_meta_language"
```

Pros:
- Matches the issue's "everything is a link" and "part and whole" framing most
  directly.
- Every assertion is itself divisible and annotatable.
- Can unify formal types, natural-language meanings, source records, and
  runtime reasoning steps.

Cons:
- Highest implementation cost.
- Existing code expects direct `Meaning`, `Lexeme`, and `WordForm` structures,
  so a full switch would require staged compatibility layers.

Best use:
Use this as the long-term normalization target after direct facets and
statement provenance are in place.

## Recommendation

Use a hybrid:

1. Keep direct recursive facets as the compact seed authoring surface.
2. Add word-form facets immediately so surfaces no longer depend only on English
   `description` text.
3. Add reified statement records for source imports, conflicts, ranks, and local
   overrides.
4. Migrate inline word forms toward OntoLex-style lexical entries and senses in
   batches.
5. Normalize mature records into typed links/holons when the compatibility API
   is ready.

## Hardcoded English Audit

Current seed audit after this review iteration:

| Item | Count |
| --- | ---: |
| Meaning records in `data/seed/meanings*.lino` | 416 |
| Meaning `gloss` strings | 416 |
| Word forms | 4,416 |
| Word `description` strings | 4,416 |
| Word-form `facet` blocks | 168 |
| Runtime word-form facet links exposed by parser | 8,888 |

The English `gloss` and `description` fields are still present and should be
treated as transitional human annotations, not as the final semantic model. This
PR adds the first executable migration path: every parsed word form now exposes
meaning-linked `notation -> word_surface` and `denotation -> parent meaning`
facets derived from the seed's existing `lexeme`/`word` nesting. Authored
word-form facets can add more links, and the new semantic-meta and lexical-meta
seed clusters use that shape for explicit `part_of_speech` data. Tests require
all exposed word-form facets to resolve as meanings.

## Source Model References

- Wikidata statements, qualifiers, references, and ranks:
  https://www.wikidata.org/wiki/Help:Data_model
- Wikidata lexicographical data:
  https://www.wikidata.org/wiki/Wikidata:Lexicographical_data/Documentation
- RDF 1.2 triple terms and reification:
  https://www.w3.org/TR/rdf12-concepts/
- SKOS Reference:
  https://www.w3.org/TR/skos-reference/
- OntoLex-Lemon:
  https://www.w3.org/2016/04/ontolex/
