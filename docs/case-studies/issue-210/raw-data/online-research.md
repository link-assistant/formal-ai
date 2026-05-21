# Issue 210 — Online Research Notes

Collected on 2026-05-21 for
<https://github.com/link-assistant/formal-ai/issues/210>.

## Sources

- **Wiktionary: Russian `яблоко`** —
  <https://en.wiktionary.org/wiki/%D1%8F%D0%B1%D0%BB%D0%BE%D0%BA%D0%BE>
  records the Russian noun `яблоко` with the English gloss `apple`.
- **Wiktionary: Russian `добрый`** —
  <https://en.wiktionary.org/wiki/%D0%B4%D0%BE%D0%B1%D1%80%D1%8B%D0%B9>
  records the Russian adjective `добрый` with the English gloss `good`.
  The issue phrase uses the neuter nominative/accusative form
  `доброе`, matching neuter `яблоко`.
- **English Wiktionary: `good`** —
  <https://en.wiktionary.org/wiki/good> documents the English target
  adjective and its multilingual translation tables.
- **English Wiktionary: `what is it`** —
  <https://en.wiktionary.org/wiki/what_is_it> demonstrates that
  short question phrases are not consistently represented as complete
  multilingual translation entries; phrase coverage is sparse.
- **Wikidata lexicographical data** —
  <https://www.wikidata.org/wiki/Wikidata:Lexicographical_data>
  documents language-independent lexeme and sense modeling used by the
  Rust translation pipeline when a Wikidata sense is available.
- **MediaWiki parse API** —
  <https://www.mediawiki.org/wiki/API:Parsing_wikitext> documents the
  `action=parse&prop=wikitext` API used by `src/translation/wiktionary.rs`
  to fetch and parse Wiktionary translation tables.

## Findings

- The reported phrase `доброе яблоко` is compositional: a Russian
  adjective form plus a Russian noun. Search surfaced authoritative
  entries for the component words, but not a reliable complete phrase
  entry. The existing pipeline can parse lexicalized translation-table
  entries, but it needs a deterministic fallback for short phrases when
  the knowledge-base lookup returns no candidate.
- The prompts `кто ты такой` and `что это такое?` are quoted source
  text inside an explicit `Переведи ... на английский` request. They
  must be treated as source surfaces even though the same words can
  trigger identity/capability handlers when they are the whole prompt.
- The browser worker cannot rely on live Wiktionary/Wikidata requests
  for this path, so the web runtime needs a CORS-safe offline fallback
  mirroring the Rust behavior for the affected prompts.

## Solution Implications

- Keep the Wiktionary/Wikidata pipeline as the canonical source of
  translation candidates.
- Run translation intent before broad identity/capability handlers when
  the user explicitly starts with a translation verb.
- Add a late, traceable `compositional:ru->en:<surface>` fallback for
  short Russian-to-English phrases only after the external-data-backed
  pipeline produces no candidates.
- Mirror the same precedence and fallback in `src/web/formal_ai_worker.js`
  so the GitHub Pages demo behaves like the Rust CLI.
