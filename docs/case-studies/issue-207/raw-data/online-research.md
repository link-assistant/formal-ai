# Issue 207 — Online Research Notes

This file collects external references used to ground the case study and
implementation plan. Each link is captured in a way that points to the
authoritative documentation page so the formal-ai pipeline can later
re-fetch it on demand. Visit dates are recorded in the timeline.

## Wikipedia, Wikidata, and Wiktionary as a translation back-end

- **Wikipedia REST page summary** —
  <https://en.wikipedia.org/api/rest_v1/page/summary/{title}> exposes
  CORS-enabled summaries with `displaytitle`, `description`, and the
  language-link table (`langlinks`) used to recover a single-token
  translation for an entity.
- **Wikipedia interlanguage links** —
  <https://en.wikipedia.org/w/api.php?action=query&prop=langlinks> returns
  every translated title for a given page. This is the cheapest entity
  translation path: source-language title → target-language title.
- **Wikidata SPARQL endpoint** — <https://query.wikidata.org/sparql>
  resolves a Q-id label across every Wikidata language. Each label in
  the `rdfs:label` or `skos:altLabel` list is a candidate surface form
  for a deformalized rendering.
- **Wikidata entity service** —
  <https://www.wikidata.org/wiki/Special:EntityData/{qid}.json> returns
  the same label set as a structured JSON document without requiring
  SPARQL.
- **Wiktionary REST definition API** —
  <https://en.wiktionary.org/api/rest_v1/page/definition/{word}> returns
  per-language definition lists, including a `translations` section in
  the source-language headword block. This is the route the formal-ai
  browser worker already uses for definition fusion (`fetchWiktionaryEntry`
  in `src/web/formal_ai_worker.js`).
- **Wikidata Lexemes** —
  <https://www.wikidata.org/wiki/Wikidata:Lexicographical_data> publishes
  lexeme-level translations including grammatical features (case,
  tense, formality). For lexicalized phrases like `how are you?` this is
  the canonical machine-readable source.

## Background reading on the formalize → deformalize pipeline

- **Wikidata's role as an intermediate language** — Section "Translation
  Between Languages" of `ARCHITECTURE.md` already documents the project's
  formalize → re-render pipeline. The same idea is the central motivator
  in the literature on the Abstract Wikipedia and Wikifunctions project.
  - <https://meta.wikimedia.org/wiki/Abstract_Wikipedia>
  - <https://meta.wikimedia.org/wiki/Abstract_Wikipedia/Architecture>
  These pages describe a renderer that takes language-independent
  "Constructors" and renders them into a target natural language.
- **Natural Semantic Metalanguage (NSM)** —
  <https://en.wikipedia.org/wiki/Natural_semantic_metalanguage>. The
  hand-curated 65-prime lexicon supplies a complementary fallback when a
  meaning has no direct Wikidata anchor.

## Translation casing conventions

- English style guides (Chicago Manual of Style 5.10; Garner's Modern
  English Usage, entry "Capitalization") agree that sentence-initial
  capitalization is required in formal prose but not in mid-sentence
  quoted fragments. Russian style follows the same rule (Розенталь,
  «Справочник по правописанию и стилистике», §3). Mid-sentence quoted
  fragments preserve their original capitalization. This is the source
  of the requirement in issue 207 that a lowercase Russian quote
  `как у тебя дела?` translates to a lowercase English `how are you?`.
- The same logic applies to terminal punctuation: when the source
  fragment ends in `?`, `!`, or `.`, the target should end with the
  matching marker; when the source has no terminal punctuation, the
  target should not invent one.

## Existing components reused

- `src/summarization/mod.rs` already implements the `formalize` →
  `summarize` → `deformalize` pipeline (R197). The translation handler
  reuses the same vocabulary so the implementation stays consistent.
- `src/web/formal_ai_worker.js` already integrates Wikipedia, Wikidata,
  and Wiktionary through `fetchProviderJson`, `fetchWiktionaryEntry`,
  and the multilingual definition fusion handler.
- `src/solver_helpers.rs::canonical_meaning_token` collapses surface
  forms to canonical tokens — the registry every translation pair
  hashes through. Extending this registry is the smallest change that
  unlocks "translate not only one hardcoded meaning, but all of them"
  without breaking determinism or offline behavior.

## Existing libraries that solve adjacent problems

- **`link-foundation/lino-i18n`** is already used for browser UI
  translations. Its key/value catalog format is the right shape for the
  meaning → multilingual-surface registry.
- **`lino-objects-codec`** provides the canonical Links Notation reader
  and writer the project relies on for every other deterministic
  pipeline.
- **`link-calculator`** delegates math/unit/currency reasoning through
  the same formalize-then-deformalize idea: a numeric expression is
  formalized, evaluated, and re-rendered. Translation follows the same
  pattern.
