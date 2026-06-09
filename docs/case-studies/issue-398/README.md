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
required facets to the `link` root. A later PR review comment supplied a
Links-Theory root draft; this PR now seeds that draft as
`data/seed/meanings-links-root.lino`, including `reference`, `link_action`,
defined connectives, self-equations, quantity primitives, and the
one-symbol-one-meaning `bank_river`/`bank_money` split. The deeper import and
backfill work remains data migration work over the same shape: source responses
should be cached as `.lino`, converted into meanings, then linked through facets
in small reviewable batches.

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
- `meaning-data-model-alternatives.md` - five alternative meaning data models
  requested during PR review, plus the hardcoded-English seed audit and the
  follow-up root-seed note.

## Timeline

| Time (UTC) | Event |
| --- | --- |
| 2026-06-06T19:15:52Z | Issue #398 opened with the recursive-meaning and semantic meta-language request. |
| 2026-06-06T19:28:04Z | Issue updated before PR work began. No issue comments were present when archived. |
| 2026-06-06T19:42:54Z | Draft PR #399 opened from `issue-398-b349f91c312f`. |
| 2026-06-06 | Repository and online research collected under this case-study directory. |
| 2026-06-06T20:37:50Z | PR review feedback rejected the first pass as too shallow, requested five alternative data-model variants, and called out remaining hardcoded English seed descriptions. |
| 2026-06-06T20:59:26Z | First review follow-up added word-form facets and documented the five variants. |
| 2026-06-06T23:03:09Z | New PR feedback supplied a concrete self-defining Links Notation root and one-symbol-one-meaning construction rules. |
| 2026-06-06 | This follow-up converted that draft into a seed cluster, root/self-equation tests, and refreshed PR metadata archives. |

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

The review follow-up made the remaining gap explicit: the seed still has 416
meaning glosses and 4,416 word descriptions as scalar text. This PR now treats
those fields as transitional human annotations. The parser derives
meaning-linked notation and denotation facets for every parsed word surface, and
explicit word-form facets add part-of-speech data for the semantic and lexical
meta-language clusters. The five evaluated model variants are documented in
`meaning-data-model-alternatives.md`.

The second review follow-up made the root-shape requirement concrete. The seed
now has a compact Links-Theory root cluster with 48 multilingual meanings:
`reference`, `reference_action`, `link_action`, `any_of_reference`,
`repeatable_from_zero`, defined connectives (`of`, `from`, `to`, `and`), the
split senses of `is`, quantity primitives (`amount`, `size`, `count`, `extent`),
and `self_equation` as a semantic facet kind. The existing `link`, `type`,
`quantity`, `zero`, and `one` meanings point into that cluster, so the new root
does not sit beside the ontology; it is part of the same closed graph.

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
| R398-11 | Represent the concrete Links-Theory root draft from PR feedback as seed data. | Existing `Meaning` records, `defined_by` closure tests, and semantic facets. | Added `data/seed/meanings-links-root.lino` and embedded it in runtime/source-test seed registries. | Expand the cluster toward typed holon/reified-statement records after compatibility APIs exist. |
| R398-12 | Keep self-referential primitives as structured self-equations, not prose synonyms. | The facet parser can attach arbitrary meaning-backed facet kinds. | Added `self_equation` as a facet kind and tests for `type`, `not`, and `same`. | Later proof work can check these fixed points with relative-meta-logic instead of only asserting seed shape. |
| R398-13 | Enforce one-symbol-one-meaning sense splitting for ambiguous words. | Existing multilingual lexical forms and word-form facets. | Seeded `one_symbol_one_meaning`, `sense_split`, `bank_river`, and `bank_money`; tests assert there is no ambiguous bare `bank` meaning. | Add migration scripts that detect ambiguous surfaces and propose split symbols with source evidence. |

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
- Added word-form semantic facets so lexical surfaces can link their notation,
  denotation, and part of speech to seed meanings.
- Derived `notation -> word_surface` and `denotation -> parent meaning` facets
  for every parsed word form from the existing seed structure.
- Added `Lexicon::semantic_facet_meanings` so callers can resolve facet targets
  back to `Meaning` records.
- Added generic parsing for nested seed blocks of this form:

```lino
facet "notation"
  meaning "links_notation_format"
```

- Added `data/seed/meanings-semantic-meta.lino` with the semantic-facet
  vocabulary and multilingual lexemes.
- Added `data/seed/meanings-lexical-meta.lino` with `word_surface`,
  `lexical_form`, `lexical_sense`, `part_of_speech`, `noun`, and `noun_phrase`.
- Added `data/seed/meanings-links-root.lino` with the self-defining root draft:
  references/actions, defined connectives, self-equations, quantity primitives,
  and sense-splitting examples.
- Updated `data/seed/meanings-ontology.lino` so the root `link` meaning declares
  `notation`, `annotation`, `denotation`, and `connotation` facets.
- Updated `link`, `type`, `quantity`, `zero`, and `one` so existing ontology and
  numeric roots point into the Links-Theory root cluster.
- Embedded the new seed file in both the runtime seed module and mirrored source
  tests.
- Added source tests proving facet blocks parse as meaning references and the
  root `link` declares all four required facet kinds.
- Added tests proving word-form facet blocks parse and the semantic-meta seed
  no longer relies only on English descriptions for its word surfaces.
- Added public unit tests proving the self-defining root terms exist, connectives
  and `is` senses are meaning-defined, self-equations are facet links, and the
  `bank` example is split into distinct symbols.

## Validation strategy

The reproducing test was written before the parser implementation. The pre-fix
run failed at compile time because `semantic_facet_targets` and
`semantic_facet_meanings` did not exist. After the implementation, the focused
semantic-facet test passed and the broader seed checks are part of the final
local verification for PR #399.

The second reproducing test was added from the 23:03 UTC PR feedback. Before the
new seed cluster existed, `cargo test semantic_root -- --nocapture` failed
because meanings such as `reference`, `link_action`, `self_equation`,
`bank_river`, and `bank_money` were absent. The final focused run passes those
tests and the whole seed-meaning invariant suite.

## Scalar readability fix (codepoint de-obfuscation)

Commit `b1e1bc6` had made the seed parse-valid the wrong way: human-readable
scalar values were replaced with codepoint byte-dumps such as
`answer codepoints 72 105 44 ...` (the integers spell "Hi, how may I help
you?"). That satisfied the parser but destroyed the meaning of the data — the
opposite of issue #398's goal that every value be readable and grounded.

This round restores readability for scalar values:

- All 57 `data/seed/*.lino` files had their codepoint byte-dumps converted back
  to readable quoted scalars (e.g. `text "Hi, how may I help you?"`). Zero
  byte-dumps remain in the seed.
- Each quoted value selects a delimiter (`"`, `'`, or backtick) that does not
  occur in its own text, so the inner quote never needs escaping. This works
  around a quirk in the canonical `links_notation` parser while keeping the
  surface text exactly as written.
- Space-significant translation markers stay quoted so leading/trailing
  whitespace is preserved (for example `text "translate … to "`,
  `text " का "`, `text " में अनुवाद"`).
- All five parsers were updated to decode the three delimiters in sync: the
  Rust runtime (`src/seed/parser.rs`), the web runtime
  (`src/web/seed_loader.js`), the e2e parser
  (`tests/e2e/scripts/lino-seed-parser.mjs`), the embedded worker fallback in
  `src/web/formal_ai_worker.js`, and the hand-maintained `tests/source` mirror.
- `surface_text` in `src/seed/meanings.rs` now reads the readable `text` child
  first and only falls back to decoding `codepoints` for legacy data.

### CI guards added this round

- **Rule 1 — no codepoint byte-dumps in seed:**
  `tests/unit/data_files.rs::seed_lino_files_have_no_codepoint_byte_dumps`
  fails the build if any seed file reintroduces a `codepoints` or
  `unformalized-raw` byte-dump (bare integer runs outside quoted spans).
- **Rule 7 — no inline unit tests in `src/`:**
  `tests/unit/ci-cd/source_test_placement.rs::src_has_no_inline_unit_tests`
  walks `src/**/*.rs` and fails if a real `#[test]`, `#[cfg(test)]`, or
  `mod tests` appears (string-literal occurrences are ignored by anchoring at
  line start). Tests belong under `tests/`.
- **Rule 3 — no pipe-packed multi-values in seed:**
  `tests/unit/data_files.rs::seed_lino_values_never_pipe_pack_multi_values`
  fails on *any* `|` in a seed value except the exempt `code` field, so a
  multi-value can never again be packed as `"a|b|c"` instead of a real link
  list `("a" "b c" d)`.

### Defect 2 — pipe-packed multi-values (resolved this round)

`|`-separated values now use the canonical reference-list form
`keyword ("a" "b c" d)` everywhere in `data/seed/*.lino` (`supported_languages`,
`tasks`, `languages`, `inputs`, `outputs`, aliases, …). `code` listings — which
legitimately contain `|` (Rust closures `|x|`, shell pipes) — are the sole
exemption. All four LiNo parsers were taught to tokenize quoted scalars that may
contain spaces (`split_reference_tokens` in `src/seed/parser.rs`,
`splitReferenceList` in the web and e2e parsers, and the migration tooling). A
new `formal_ai::supported_languages()` accessor reads the `agent-info.lino`
reference list, replacing ad-hoc `split('|')` calls across the test suite. The
data mutation is reproducible by algorithm via `scripts/migrate-pipe-lists.rs`
(std-only). CI rule 3 (above) guards against regressions.

### Deferred to follow-up issues

The remaining defects from the PR review are intentionally out of scope for this
round and tracked below:

- **Defect 3 — own-slug naming:** internal slugs should be hyphenated full
  English words (`links-root`, not `links_root`); external `Q…/L…/P…` ids are
  fine. Deferred; CI rule 4 not yet added.
- **Defect 4 — full recursive grounding closure:** every core meaning should be
  recursively defined/grounded to closure. Deferred; CI rule 5 not yet added.
- **Rule 6 — no hardcoded domain-data string literals in `src/`:** deferred.

## PR review standards round (comment 4663407299, 2026-06-09)

The 2026-06-09 review tightened the data-quality bar and asked that every rule
be applied tree-wide through re-runnable migrations and locked by a
directory-walking CI check. The standards are recorded in `REQUIREMENTS.md`
(R278-R283) under the governance rule *latest requirement overrides any earlier
one*. Resolutions this round:

- **Empty-redefinition fields removed (R278).** Semantic facets were written as
  a `facet <kind>` wrapper whose single child was an empty colon redefinition
  (`word_surface:`) — exactly the valueless `concept:` shape the review bans.
  `scripts/migrate-empty-facet-fields.rs` collapses every such block into the
  native Links Notation `subject predicate` form (`notation word_surface`)
  across the whole `data/seed` tree and regenerates the browser worker embed.
  `src/seed/meanings.rs` now reads both forms and de-duplicates targets. The
  tree-walking guard `seed_lino_files_have_no_empty_redefinition_fields`
  (`tests/unit/data_files.rs`) fails on any empty-bodied colon field that has no
  deeper-indented child, with no hard-coded filename.

- **`data/overrides/` grounding override layer added (R279).** A new layer sits
  beside `data/cache/` with the same per-id structure
  (`data/overrides/wikidata/{entity,property,lexeme}/<id>.lino`). Resolution is
  `(cache or live API) then overrides`: `formal_ai::seed::resolve` decorates a
  cached record with the override's facts (override wins on conflict, missing
  keys are appended, untouched sections survive). Each override records **why**
  it exists in a `reason "..."` line. `tests/unit/overrides.rs` walks the whole
  tree and fails when an override references an id with no checked-in cache
  record, omits its reason, carries no facts, or is **redundant** — repeating a
  value the cache already holds. The redundancy check makes the layer
  self-pruning: once a cache refresh (or live API) catches up to upstream, the
  override must be deleted or CI stays red. The seeded example supplies the
  Hindi (`hi`) label that Wikidata is missing for the KISS principle (Q131560).

- **Tree-wide enforcement and migration-first discipline (R280, R281).** Both
  new guards walk their entire trees and assert on relative paths, so a new file
  cannot bypass a rule. Both data mutations are reproducible by algorithm
  (`scripts/migrate-empty-facet-fields.rs`, `scripts/clean-seed-readability.rs`)
  rather than hand-edited.

- **Lossless cache codec and native-node migration (R283, R284).** The
  JSON ↔ Links Notation cache codec (`formal_ai::json_lino`) now losslessly
  round-trips the *entire* source snapshot (`forms`, `senses`, `claims`, every
  key), and the circular round-trip test was replaced by one that rebuilds the
  full original JSON from the lino and asserts key-for-key equality with the raw
  `.json` (`wikidata_lino_cache_rebuilds_full_json_losslessly`). Empty
  arrays/objects/nulls are never emitted, and Wiktionary snapshots are
  pretty-printed multi-line. Separately, every meaning header migrated from the
  YAML-style trailing-colon form (`monday:`) to a native Links Notation node
  (`monday`) tree-wide via `scripts/migrate-empty-redefinition-fields.rs`,
  removing all 428 empty colon redefinition fields; the transform is
  parse-equivalent and `seed_lino_files_have_no_empty_redefinition_fields`
  enforces the reviewer's exact `^\s*[\w-]+:\s*$` regex.

- **External grounding (R282) — verified pipeline, monotonic backfill.**
  Source-grounded meanings carry `grounded-in <Qid>` links whose recursive
  closure is verified against checked-in cache records
  (`wikidata_cache_records_cover_recursive_grounding_closure`,
  `seed_and_source_wikidata_ids_have_checked_in_cache_records`). The backfill is
  now driven by `scripts/ground-meanings.rs`, a re-runnable, **self-verifying**
  pipeline: for each curated `(slug, id, expected-label-token)` it fetches
  `Special:EntityData/<id>.json`, trims it to the cache convention, and grounds
  the meaning **only if** the fetched entity's labels actually contain the
  expected concept token. That guard is essential — a hand-assigned id is wrong
  more often than not (`Q206` resolves to "Stephen Harper", not "seven";
  `Q170043` to "perfect number", not "modulo"), and the verifier refuses every
  such mismatch rather than injecting a plausible-but-wrong anchor. The first
  batch grounded 37 common-vocabulary meanings (weekdays, arithmetic operations,
  currencies, length/mass/time/data units, temperature, math functions, core
  quantities), raising checked-in coverage from 18 to 55 `grounded-in` anchors,
  each with its source snapshot under `data/cache/wikidata/entity/`.

  Rather than a fail-until-complete grounding gate (which would only turn CI red
  without protecting an invariant while the corpus import proceeds), coverage is
  protected by a **monotonic ratchet**, `grounded_meaning_coverage_does_not_regress`
  (`tests/unit/data_files.rs`), matching the repo's established benchmark-ratchet
  pattern: the grounded-meaning floor (54) can only rise, so grounding is
  append-only and every batch is locked in. Grounding the long tail of
  domain-composite meanings (e.g. `program_task_fizzbuzz`,
  `feature_capability_web_search`) — which have no single Wikidata entity and
  must be anchored through the override layer (R279) — and sourcing full
  per-language word forms from Wikidata lexemes remain the primary follow-up
  below.

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
