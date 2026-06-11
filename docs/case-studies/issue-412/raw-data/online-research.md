# Online research — issue #412

Collected 2026-06-11 while scoping the conversational numeric-list follow-up
defect and the broader generalization directions the issue requests.

## External knowledge sources suggested by the issue

The issue asks us to treat several public "how do I do X in language Y" corpora
as external APIs, cache the most popular cells, and merge them into the existing
views. Findings on their machine-readability:

### Wikifunctions (https://www.wikifunctions.org)

- Has a real, documented REST/Action API. A `Z7`/Function call is POSTed and the
  orchestrator returns a `Z22`/Evaluation result (the value plus evaluation
  metadata). Reference: <https://api.wikimedia.org/wiki/Wikifunctions_API/Reference>
  and <https://www.mediawiki.org/wiki/Extension:WikiLambda/API>.
- Relevance: a *language-agnostic* function evaluator — it computes the result of
  e.g. "sort this list", but it does not, by itself, emit idiomatic source code
  in a chosen target language. Useful as a **result oracle** (cross-check our
  in-solver computation) rather than as a code generator.
- CORS / availability through `api.wikimedia.org` is the same Wikimedia surface
  we already consume for Wikidata/Wiktionary, so it fits the existing cache shape
  (`data/cache/<api>/<entity>/…`).

### Rosetta Code (https://rosettacode.org)

- No first-class REST API; it is a MediaWiki instance, so its `api.php` (and the
  category `Programming_Tasks`) is the practical scraping surface. Each task page
  holds idiomatic implementations per language — a natural **code corpus** for
  the "give me the code" half of the request.

### Hello World Collection (http://helloworldcollection.de) / Stack Overflow

- Hello World Collection is a static catalogue (no API); a one-shot snapshot per
  language is enough. Stack Overflow exposes the Stack Exchange API but is rate
  limited and license-encumbered; lower priority than the two above.

## Implication for the cache cap the issue specifies

The issue requires: *never cache everything — at most 1%, or 512 items if 1% is
smaller, per data set / API / merged topic.* This matches the existing
`data/cache/<api>/…` discipline (trimmed per-entity snapshots checked in, ratchet
tests guarding coverage). Any future Wikifunctions/Rosetta integration should
reuse that pattern and add a per-source cap constant rather than inventing a new
mechanism.

## meta-language is already the coding engine

The issue asks us to "make sure we actually use
<https://github.com/link-foundation/meta-language> for all coding manipulation
tasks." The numeric-list handler already routes every generated program through
the meta-language CST engine — the live trace for the reproduction now contains:

```
synthesis:cst_engine meta_language
synthesis:cst_tree … source_repository https://github.com/link-foundation/meta-language …
```

so the established coding path is already meta-language-backed. The remaining
generalization work is to widen *which* tasks reach that path and to ground the
results against the external oracles above — captured as future work below.

Sources:
- [Wikifunctions API Reference](https://api.wikimedia.org/wiki/Wikifunctions_API/Reference)
- [Extension:WikiLambda/API](https://www.mediawiki.org/wiki/Extension:WikiLambda/API)
- [Abstract Wikipedia / Function Evaluation for Wikifunctions](https://meta.wikimedia.org/wiki/Function_Evaluation_for_Wikifunctions)
