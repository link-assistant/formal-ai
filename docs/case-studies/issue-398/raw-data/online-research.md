# Online research - Issue #398 semantic meta-language

Fetched on 2026-06-06. The issue asks for comparison with popular external
knowledge and dictionary sources, and for a plan to convert source responses to
Formal AI views. This note records the source model implications, not a bulk
download.

## Sources checked

| Source | Relevant model | Import implication |
| --- | --- | --- |
| [Wikidata data model](https://www.wikidata.org/wiki/Help:Data_model) | Items and properties carry labels, descriptions, aliases, sitelinks, statements, qualifiers, references, ranks, and values. | Formal AI meanings should map Wikidata items/properties to denotation facets, while references become source/cached-response meanings. |
| [Wikidata database download](https://www.wikidata.org/wiki/Wikidata:Database_download) | Wikidata publishes entity dumps suitable for offline import. | Importers should save raw or normalized chunks as `.lino` source responses before deriving meaning facets. |
| [Wikidata lexicographical data](https://www.wikidata.org/wiki/Wikidata:Lexicographical_data/Documentation) | Lexemes contain forms and senses; forms and senses can also have statements. | Formal AI word forms need a later facet layer for form, sense, part-of-speech, and statement evidence. |
| [Wikimedia dumps](https://dumps.wikimedia.org/backup-index.html), [English Wikipedia dumps](https://dumps.wikimedia.org/enwiki/), [English Wiktionary dumps](https://dumps.wikimedia.org/enwiktionary/) | Wikimedia publishes periodic dump indexes for wiki content. | Wikipedia and Wiktionary import should be chunked and cached, not embedded directly in startup seed files. |
| [WordNet documentation](https://wordnet.princeton.edu/documentation) and [WordNet input format](https://wordnet.princeton.edu/documentation/wninput5wn) | WordNet organizes word senses into synsets with semantic relation pointers and glosses. | Synsets map well to meanings/concepts; pointer symbols map to relation meanings; glosses map to annotation facets. |
| [SKOS Reference](https://www.w3.org/TR/skos-reference/) | SKOS models concepts, concept schemes, preferred/alternate labels, and broader/narrower/related semantic relations. | SKOS can provide a comparison vocabulary for taxonomy/thesaurus imports and for broad/narrow/related source alignment facets. |
| [OntoLex-Lemon](https://www.w3.org/2016/04/ontolex/) | OntoLex models lexical entries, forms, lexical senses, lexical concepts, and ontology references. | OntoLex is a strong guide for extending `WordForm` into richer lexical-entry and sense records without hardcoding one language. |
| [OntoLex lexicography module](https://www.w3.org/2019/09/lexicog/) | The lexicography module groups lexical entries into lexicographic records and supports usage examples and translations. | Useful for future dictionary-style imports where one human dictionary entry contains several formal senses. |
| [RDF 1.2 Concepts](https://www.w3.org/TR/rdf12-concepts/) | RDF 1.2 includes triple terms and reification so assertions can themselves be described. | Useful as a reference point for later statement records where a source fact, qualifier, or override must itself become a meaning-linked object. |

## Findings

Wikidata is the closest match for language-independent denotation: its items and
properties already separate identifiers from multilingual labels and can carry
references. The current `wikidata` scalar field in Formal AI is therefore a good
anchor, but it should eventually become a structured source/facet relation so
qualifiers, references, ranks, and local overrides remain inspectable.

Wikidata lexicographical data, OntoLex-Lemon, and WordNet all separate surface
forms from senses. That supports the issue's request to give words, senses, and
parts of speech the same semantic treatment as meanings. The review follow-up
added this bridge to `WordForm`: word forms now expose derived notation and
denotation facets from seed structure and can carry authored facets for
part-of-speech or richer lexical metadata, while a future importer can still
migrate them to full lexical-entry and lexical-sense records.

Wikipedia and Wiktionary dumps are too large for startup seed embedding. They
should be treated as external source corpora with raw response snapshots,
normalized `.lino` cache files, and derived semantic records. This matches the
existing repository direction: the seed remains small, while source-cache and
translation-cache data can expand on demand.

SKOS provides a compact standard vocabulary for concept schemes and thesaurus
relations. Its broader/narrower/related distinction is a useful external
alignment model for Formal AI's future "exact denotation versus broad/narrow
conceptual neighbor" comparisons.

## Recommended importer shape

1. Fetch or read source dump chunks with explicit source URL, fetch time, and
   content hash.
2. Store original source responses as `.lino` source-response records, not only
   as transient JSON/XML.
3. Derive meaning records and facet links from those cached responses.
4. Mark derived links with provenance and keep local overrides separate from
   source facts.
5. Keep generated files below repository line-size limits and split by source,
   language, and domain.
