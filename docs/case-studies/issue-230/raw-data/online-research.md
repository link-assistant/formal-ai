# Online Research

Issue #230 asks for the phrase:

```text
Найти синонимы или примеры согласования
```

to translate from Russian to English.

## References

- Wiktionary's Russian entry for `согласование` lists the genitive form
  `согласования` and gives the English gloss `concordance (agreement)`:
  <https://en.wiktionary.org/wiki/%D1%81%D0%BE%D0%B3%D0%BB%D0%B0%D1%81%D0%BE%D0%B2%D0%B0%D0%BD%D0%B8%D0%B5>
- Wiktionary's English entry for `agreement` includes a linguistics /
  grammar sense and lists `concord` / `concordance` as synonyms for that
  sense:
  <https://en.wiktionary.org/wiki/agreement>
- Wikipedia's `Agreement (linguistics)` article describes agreement as
  grammar rules where parts of a sentence are inflected according to
  attributes of other parts, and explicitly discusses Slavic/Russian
  agreement:
  <https://en.wikipedia.org/wiki/Agreement_(linguistics)>
- Reverso Context documents a practical fallback for phrases without
  exact dictionary matches: break shorter text into smaller pieces and
  translate each piece. That supports a deterministic compositional
  fallback after exact Wiktionary/Wikidata lookup fails:
  <https://context.reverso.net/%D0%BF%D0%B5%D1%80%D0%B5%D0%B2%D0%BE%D0%B4/about>
- Wikidata's Lexicographical data documentation explains that senses in
  different languages can be considered translations when they point to
  the same language-neutral item via `item for this sense (P5137)`.
  This supports the project architecture of formalizing source surfaces
  into a meaning id before deformalizing them into a target surface:
  <https://www.wikidata.org/wiki/Wikidata:Lexicographical_data/Documentation/Senses>
- MediaWiki documents `action=parse` as the API used to parse a page and
  obtain parser output. The existing pipeline uses that family of
  endpoints to fetch Wiktionary wikitext before extracting translation
  tables:
  <https://www.mediawiki.org/wiki/Special:MyLanguage/API:Parsing_wikitext>
- Wiktionary's `{{t}}` / `{{t+}}` documentation describes translation
  templates and also distinguishes cases with no equivalent or no
  attested translation. That reinforces the product invariant that a
  missing candidate is not itself a translation surface:
  <https://en.wiktionary.org/wiki/Template:t%2B>
- Wiktionary's `{{trans-top}}` documentation describes translation
  tables as sense-specific and keyed by gloss / sense id, which matches
  the pipeline's sense-block selection instead of flattening all
  candidates blindly:
  <https://en.wiktionary.org/wiki/Template:trans-top/documentation>

## Applied Interpretation

`согласования` is genitive singular of `согласование`. In the reported
phrase, `примеры согласования` is therefore rendered as `examples of
agreement`, not as the flat word sequence `examples agreement`.

The resulting target surface is:

```text
Find synonyms or examples of agreement
```

When no source Wiktionary block, reverse lookup, Wikidata lexeme join, or
compositional fallback produces a candidate, the result is a translation
gap rather than a target-language string. The Rust handler and browser
worker now preserve that distinction structurally: candidate surfaces are
quoted as translations, while misses produce explicit user-facing gap
text and `translation_gap:<surface>` evidence.
