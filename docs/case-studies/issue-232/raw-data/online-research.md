# Issue 232 Online Research

Collected on 2026-05-22.

## Russian Wikipedia Summary

Source:
<https://ru.wikipedia.org/api/rest_v1/page/summary/%D0%A1%D1%83%D1%89%D0%B5%D1%81%D1%82%D0%B2%D0%BE>

Saved payload: `ruwiki-summary-suschestvo.json`

Findings:

- The exact page title is `Существо`.
- The summary API marks the page as `type: "disambiguation"`.
- The extract contains definition-style entries for the living-organism
  sense and the essence/sustnost sense.
- The desktop page URL is
  <https://ru.wikipedia.org/wiki/Существо>.

## Russian Wikipedia Parse API

Source:
<https://ru.wikipedia.org/w/api.php?action=parse&page=%D0%A1%D1%83%D1%89%D0%B5%D1%81%D1%82%D0%B2%D0%BE&prop=text&format=json&formatversion=2&redirects=1&origin=*>

Saved payload: `ruwiki-parse-suschestvo.json`

Findings:

- The parsed page HTML includes the same definition-style entries from
  the summary extract.
- The page also includes a `В культуре` section with entries such as the
  2011 Dolphin album and the 1982 horror film.
- The implementation extracts list item text before see-also/reference
  sections so it can render the page meanings without MediaWiki markup.

## Russian Wikipedia Search

Source:
<https://ru.wikipedia.org/w/rest.php/v1/search/page?q=%D0%A1%D1%83%D1%89%D0%B5%D1%81%D1%82%D0%B2%D0%BE&limit=5>

Saved payload: `ruwiki-search-suschestvo.json`

Findings:

- The top search page is the exact `Существо` page.
- Search also returns related pages such as Marvel and media entries.
- The issue is not caused by Wikipedia search missing the exact page; it
  is caused by the direct summary disambiguation skip.

## Wikidata Search

Source:
<https://www.wikidata.org/w/api.php?action=wbsearchentities&format=json&origin=*&type=item&limit=5&language=ru&search=%D1%81%D1%83%D1%89%D0%B5%D1%81%D1%82%D0%B2%D0%BE>

Saved payload: `wikidata-search-suschestvo.json`

Findings:

- Wikidata ranks `Q729` (`Animalia`) first because `существо` is an
  alias.
- That first exact alias is enough for the worker's Wikidata concept
  fallback, which explains the reported answer.
- Wikidata is useful as a fallback, but it should not override an exact
  Russian Wikipedia page that already contains definition-style entries.
