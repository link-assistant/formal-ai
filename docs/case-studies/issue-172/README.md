# Issue 172: Procedural How-To Discovery

## Prompt Shape

The reported examples are procedural questions such as:

- `How to make tea?`
- `How to prepare fried potatoes?`

The important structure is not the examples themselves. The solver needs to
recognize `how to X Y`, extract the action `X` and related object `Y`, and
record a source-backed discovery path for arbitrary legal tasks.

## Source Research

wikiHow exposes a MediaWiki API at `https://www.wikihow.com/api.php`.
The API supports CORS for JSON endpoints such as `siteinfo`, `opensearch`, and
`parse`.

Useful findings:

- `action=parse&page=Make-Tea&prop=text|sections|displaytitle&format=json&origin=*`
  returns the article HTML and section data for a known page.
- `opensearch` is prefix-oriented and did not reliably discover full how-to
  titles for example queries such as `make tea`.
- `wikiHowTo?search=...` returns useful HTML search results, but that endpoint
  is not the same CORS-readable JSON API surface.
- Unknown candidate pages return MediaWiki errors, so the solver must fall back
  to web search and then fetch candidate pages only when they contain explicit
  ordered or instructional steps.

## Implemented Path

The Rust solver now recognizes procedural prompts, extracts task/action/object
fields, and records this deterministic trace:

1. Wikipedia context stage.
2. Wikidata entity/action/object stage.
3. wikiHow parse API candidate.
4. Web-search fallback with the existing provider priority.
5. Recursive fetch check that accepts only pages with explicit steps.

The browser worker mirrors the same intent. When CORS `fetch` is available, it
tries the wikiHow parse API candidate first and extracts a short step list. If
that candidate misses, it delegates to the existing reciprocal-rank-fusion web
search path.

## Verification

Regression tests were added for:

- `How to make tea?`
- `How to prepare fried potatoes?`
- `How can I calibrate a torque wrench?`

The third case keeps the handler honest by verifying that it is not memoized to
the issue examples.
