# Case study - Issue #398: Recursive semantic meta-language for meanings

## Summary

Issue #398 asks Formal AI to make every meaning explainable through other
meanings, rather than through hardcoded language constants or English-only prose.
The repository already had an important foundation: every seeded meaning has
`defined_by` links, every `defined_by` target must exist, and every meaning must
reach the self-rooted `link` root. The missing piece was a structured way to say
what a meaning's notation, annotation, denotation, and connotation are without
turning those into Rust enum values or free-text fields.

This PR adds that foundation. Meanings can now carry generic `facet` blocks, each
facet kind is itself a meaning, and facet targets are meaning references. The
seed adds the first semantic-facet vocabulary (`semantic_facet`, `notation`,
`annotation`, `denotation`, `connotation`, `semantic_gloss`,
`external_knowledge_source`, and `cached_source_response`) and attaches all four
required facets to the `link` root. The deeper import and backfill work remains
data migration work over the same shape: source responses should be cached as
`.lino`, converted into meanings, then linked through facets in small reviewable
batches.

## Archived data

- `raw-data/issue-398.json` - issue metadata and body captured with
  `gh issue view`.
- `raw-data/issue-398-comments.json` - issue comments captured with the
  paginated GitHub API.
- `raw-data/pr-399.json` - draft PR metadata captured before implementation.
- `raw-data/pr-399-conversation-comments.json` - PR conversation comments.
- `raw-data/pr-399-review-comments.json` - PR inline review comments.
- `raw-data/pr-399-reviews.json` - PR review records.
- `raw-data/online-research.md` - external source survey for Wikidata,
  Wiktionary/Wikipedia dumps, WordNet, SKOS, and OntoLex-Lemon.

## Timeline

| Time (UTC) | Event |
| --- | --- |
| 2026-06-06T19:15:52Z | Issue #398 opened with the recursive-meaning and semantic meta-language request. |
| 2026-06-06T19:28:04Z | Issue updated before PR work began. No issue comments were present when archived. |
| 2026-06-06T19:42:54Z | Draft PR #399 opened from `issue-398-b349f91c312f`. |
| 2026-06-06 | Repository and online research collected under this case-study directory. |

## Existing implementation

The existing meaning seed is already closer to the requested model than a
plain dictionary table:

- `data/seed/meanings-*.lino` stores language-independent meanings with
  multilingual lexical forms.
- `src/seed/meanings.rs` parses the seed into `Meaning`, `Lexeme`, and
  `WordForm` records.
- `tests/source/source_tests/seed/meanings/tests.rs` requires every meaning to
  have a slug, gloss, Wiktionary anchor, role, nonempty `defined_by` links, and
  lexical forms in `en`, `ru`, `hi`, and `zh`.
- The same tests require every `defined_by` target to resolve and every meaning
  to reach the ontology root `link`.
- External-source infrastructure already exists for other features:
  `data/translation-cache/` stores cached Wiktionary/Wikidata responses for the
  translation pipeline, and `src/solver.rs` / `src/solver_handlers/` emit
  source-cache provenance events for live source lookups.

The gap was that semantic metadata lived partly as scalar fields (`gloss`,
`wiktionary`, `wikidata`) and partly in code assumptions. There was no recursive
field where a meaning could say "my notation is this meaning" or "my annotation
is this meaning" while keeping the facet kind itself in the seed.

## Requirements and solution plan

| ID | Requirement from issue #398 | Existing components checked | Solution in this PR | Follow-up plan |
| --- | --- | --- | --- | --- |
| R398-01 | Preserve issue data and perform a deep case study with online research. | Prior case studies under `docs/case-studies/issue-*`; `gh issue view`; PR comment APIs. | Archived issue/PR/comment/review data and added this case study plus online research notes. | Continue appending future source-import experiment logs under this directory if issue #398 grows into child issues. |
| R398-02 | Prefer meaning-to-meaning descriptions for every meaning. | `defined_by` links and root-reachability tests already enforce a recursive ontology path. | Kept the existing closure guarantees and added semantic facets as another meaning-to-meaning relation. | Backfill facets for all existing meanings in small batches, starting with ontology and language-domain clusters. |
| R398-03 | Avoid hardcoded language constants and manipulate meanings in code. | Existing parser knew only structural seed terms such as `meaning`, `lexeme`, and `word`. | Code only parses a generic `facet` container. `notation`, `annotation`, `denotation`, and `connotation` are seed meanings, not Rust enum variants. | Move more handler vocabularies from code constants into seed meanings when touched by future issues. |
| R398-04 | Add notation, annotation, denotation, and connotation for meanings, words, and related semantic objects. | `Meaning` had `gloss`, `wiktionary`, `wikidata`, `defined_by`, roles, and lexemes, but no structured semantic facets. | Added `SemanticFacet`, parser support, resolution helpers, seed facet-kind meanings, and root `link` facet declarations. | Extend `WordForm` and lexical-sense data with the same generic facet pattern once the word-level source import shape is designed. |
| R398-05 | Treat everything as links and support self-reference or cycles where useful. | Existing `link` root is self-defined and tests allow cycles as long as the root is reachable. | Facets are meaning references, so they remain part of the same link-native seed network. | Add migration scripts that derive or update facets from source-cache data instead of one-off manual edits. |
| R398-06 | Merge formal types with natural-language meanings and keep unclear items as variables/ranges. | Existing ontology meanings include `type`, `entity`, `concept`, `relation`, `action`, and `property`; reasoning traces already model unresolved formalization. | The semantic-facet seed describes annotation artifacts, external sources, and cached responses as ontology meanings. | Add source-alignment records that distinguish exact denotation, broad/narrow matches, unclear mappings, and local overrides. |
| R398-07 | Compare seed data with Wikipedia, Wikidata, Wiktionary, WordNet, thesauri, reverse dictionaries, and dictionaries. | Current repo already uses seeded concepts, Wikidata/Wiktionary translation cache, web-search provider checks, and source-cache events. | Added the source survey in `raw-data/online-research.md` and modeled source/cache artifacts in the meaning seed. | Build importers around official dumps/APIs and SKOS/OntoLex-compatible mappings, then store source responses as `.lino` snapshots. |
| R398-08 | Route every data item to external sources and clearly mark overrides. | Source-cache events include source URL, fetch time, hash, refresh, cache-hit, and conflict evidence. | Added `external_knowledge_source` and `cached_source_response` meanings so this provenance can become first-class semantic data. | Convert existing cache records into `.lino` source-response meanings and add override/conflict facet meanings. |
| R398-09 | Keep startup small while supporting on-demand expansion. | `data/seed/` is intentionally reviewable and line-limited; live/cache lookups expand knowledge lazily. | Seeded only the compact meta-language foundation, not a bulk import of external corpora. | Import large corpora through chunked generated files and cache/migration scripts, with each file under the repository line-size limit. |
| R398-10 | Update vision, requirements, and review evidence. | `VISION.md`, `REQUIREMENTS.md`, changelog fragments, source tests. | Updated docs and added reproducing semantic-facet tests. | Future child issues should add requirement rows that point to each importer/backfill batch. |

## Root cause

The root cause was not a missing English definition. The seed could already
prove that meanings reduce recursively to `link`, but it could not express the
semantic role of a description as data. For example, a `gloss` was a string, not
a meaning-linked annotation artifact; `wiktionary` and `wikidata` were anchors,
not a general external-source/cached-response model; and there was no place for
notation, annotation, denotation, or connotation that could be expanded without
editing Rust.

## Implementation

- Added `SemanticFacet` and `Meaning::semantic_facet_targets` to the seed
  parser.
- Added `Lexicon::semantic_facet_meanings` so callers can resolve facet targets
  back to `Meaning` records.
- Added generic parsing for nested seed blocks of this form:

```lino
facet "notation"
  meaning "links_notation_format"
```

- Added `data/seed/meanings-semantic-meta.lino` with the semantic-facet
  vocabulary and multilingual lexemes.
- Updated `data/seed/meanings-ontology.lino` so the root `link` meaning declares
  `notation`, `annotation`, `denotation`, and `connotation` facets.
- Embedded the new seed file in both the runtime seed module and mirrored source
  tests.
- Added source tests proving facet blocks parse as meaning references and the
  root `link` declares all four required facet kinds.

## Validation strategy

The reproducing test was written before the parser implementation. The pre-fix
run failed at compile time because `semantic_facet_targets` and
`semantic_facet_meanings` did not exist. After the implementation, the focused
semantic-facet test passed and the broader seed checks are part of the final
local verification for PR #399.

## Follow-up issues worth opening

1. Backfill semantic facets for all existing ontology and domain meanings.
2. Add a `.lino` source-response cache format for Wikidata, Wiktionary,
   Wikipedia, WordNet, SKOS vocabularies, and OntoLex-compatible lexical data.
3. Add source-comparison migrations that classify mappings as exact, broader,
   narrower, related, conflicting, or locally overridden.
4. Extend word forms and lexical senses with the same facet mechanism after the
   source import shape is stable.
5. Add UI/API query paths that explain a meaning by traversing `defined_by` and
   `facet` links instead of rendering scalar glossary fields alone.
