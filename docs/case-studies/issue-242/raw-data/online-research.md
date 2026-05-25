# Online Research

Collected on 2026-05-25 for issue #242.

## Dictionary Meaning

- Cambridge Dictionary has an English entry for `digress` at
  <https://dictionary.cambridge.org/us/dictionary/english/digress>. Its
  meaning is that a speaker or writer moves away from the main subject.
- Merriam-Webster has an entry at
  <https://www.merriam-webster.com/dictionary/digress>. Its meaning is that
  someone speaks or writes about something different from the main subject.
- Wiktionary has an English page at <https://en.wiktionary.org/wiki/digress>.
  The CORS-friendly opensearch endpoint is captured in
  `wiktionary-digress-opensearch.json`.
- Wikidata entity search for `digress` returns concept-level matches for
  `digression`, including Q2383053, described as a temporary subject shift in a
  composition or speech. The captured response is
  `wikidata-digress-search.json`.

## Connectivity Notes

- Wikidata and Wiktionary are suitable for direct browser fallback because the
  existing worker calls MediaWiki APIs with `origin=*`.
- Cambridge Dictionary, Merriam-Webster, Dictionary.com, and Collins are useful
  dictionary page sources, but they should stay page/proxy targets in the
  connectivity dashboard rather than default CORS providers.
- A direct HEAD request to Cambridge returned Cloudflare 520 in this
  environment, reinforcing that dictionary pages need proxy diagnostics rather
  than direct browser fetch assumptions.

## Related Project Work

- PR #208 added the generalized Wiktionary and Wikidata translation pipeline.
- PR #222 replaced pre-seeded dictionary data with raw API-response cache
  behavior for common-noun translation.
- PR #173 tightened validation around Wikimedia term lookups.
- Older concept-lookup PRs #22 and #32 show the existing pattern: add a
  general parser shape and pin it with issue-specific regression tests.
