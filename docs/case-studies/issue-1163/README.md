# Issue #1163 — The Internet as Formal Knowledge

Issue [#1163](https://github.com/link-assistant/formal-ai/issues/1163) (E128,
part of the #1183 umbrella): web search existed, but what it returned stayed
"observations", strings the solver could show and then forget. Reasoning could
only draw on the registered sources in `data/seed/sources-registry.lino`
through six bespoke extractors, and no path fetched an arbitrary page and
handed its structure to reasoning.

The raw issue, its (empty) comment list and a projection of its timeline are
under `raw-data/`. The requirement table with per-row status is the shard
`docs/requirements/issue-1163-internet-as-formal-knowledge.md`.

## Timeline

| When (UTC) | What |
| --- | --- |
| 2026-09-29 09:16 | #1163 opened with ten numbered requirements, a design, a test plan and a definition of done; linked to the #1183 umbrella at 09:21. |
| 2026-09-29 09:34–11:03 | Cross-referenced from #1164, #1165, #1168, #1159, #1166, #1184, #1185, #1172, #1174, #1178 and #1173. |
| 2026-09-30 | `689c44c49` adds `rust/src/web_formalize.rs`, the two seeds (`page-formalization-rules.lino`, `source-trust-weights.lino`), three page fixtures and `rust/tests/unit/issue_1163_web_formalize.rs`. |
| 2026-09-30 | `fca05d36d` wires the formalizer into `source_research::learning_proposal`: every captured page's statements are emitted as `formalized_page_statement` records with URL and SHA-256. |
| 2026-10-06 | `cb5cc2523`, `922710934`, `4d392b0d3`: formatting and lint passes only. |
| 2026-10-07 | `46d45c9e1` fixes the seed loaders (records are read under the file's wrapper). |
| 2026-10-07 | `7d4f110d2` fixes the HTML walker (container tags are stepped into, comments and opaque tags skipped, close tags matched by full name); `a0e1786ee` names the comment delimiters as constants. |
| 2026-10-07 | PR #1188 (branch `qa-reasoning-coding-bulk-fixes`) cross-references the issue. |

## Requirements

The issue's ten numbered requirements are `R1163-1` to `R1163-10`, in order.
The other sections add six obligations, `R1163-11` to `R1163-16`: the gated
online Kotlin test, the `web_page` registry row with its `generic_page_v1`
dispatcher branch, the `document_formats.rs` conversion source, captured (not
reshaped) fixtures, the case-study artefacts, and the changelog fragment.

Status on this branch, from the shard:

| Status | IDs |
| --- | --- |
| Implemented, with a test | R1163-1, R1163-2, R1163-9 |
| Partial | R1163-3, R1163-5, R1163-6, R1163-7, R1163-8, R1163-14, R1163-15 |
| Not started | R1163-4, R1163-10, R1163-11, R1163-12, R1163-13, R1163-16 |

In short: the formalizer, its trust score, its store and its queries exist as
a library and are pinned offline. What is missing is the wiring that makes
them the production path: routing an unmatched need to search (R1163-4), the
registry row and dispatcher branch (R1163-12), the solver's working memory
(R1163-6), the memory query language (R1163-8), feature extraction for the
trust score (R1163-7), and the JS/TS roots (R1163-10).

## Root causes

1. **No consumer of fetched bytes as structure.** `source_fetch.rs` already
   captured pages with URL, timestamp and SHA-256, and `document_formats.rs`
   already converted documents through meta-language, but nothing turned a
   page into links. Each registered source needed its own extractor, so a
   source without one was unusable.
2. **The first walker skipped the whole document.** The original HTML walker
   took every opening tag, found its close tag, and moved the cursor past the
   close tag. For `<html>` that close tag is `</html>`, so a real page,
   wrapped in `<html><body>`, produced no blocks at all. The same search for
   `</p` also matched `</pre>`. Commit `7d4f110d2` steps into container tags,
   skips only comments and opaque tags (`script`, `style`, `template`,
   `noscript`), and accepts a close tag only when the name ends there
   (`find_close_tag`).
3. **The rule loaders read no rules.** `page-formalization-rules.lino` and
   `source-trust-weights.lino` put every record under one wrapper
   (`page_formalization_rules`, `source_trust_weights`). The loaders iterated
   the top level, which holds only that wrapper, so no mime hint, sniff rule,
   tag source or trust weight loaded: every code block got the `unknown`
   fallback and every trust score was 0. Commit `46d45c9e1` adds
   `seed_records`, which reads the children of the single wrapper (or the top
   level when a file has none), and accepts a trust feature named by its id.
4. **Library first, wiring later.** The feature commit states that the
   registry row, the search fetch loop, the memory-query predicates and the
   worker translation land with a later integration. Only the search fetch
   loop has landed since, which is why most rows are partial.

## Prior art

- **Inside this repository.** `concept_lookup.rs` (`read_glosses` and the six
  bespoke extractors), `source_fetch.rs` (`CachedSourceClient`,
  `SourceCapture`), `web_search_core.rs` (the RRF fusion constants and
  provider specs), `document_formats.rs` (meta-language document conversion)
  and the `primacy` field of the source registry, which the trust score
  reuses as a feature.
- **HTML to structure.** Browser-grade parsers (html5ever, the HTML Living
  Standard's tree builder) and content extractors (Mozilla Readability,
  trafilatura) solve the general problem. This module walks tags by hand to
  stay dependency-free; the cost is that malformed markup (unclosed `<p>`,
  nested `<pre>`) is handled less forgivingly than a real tree builder would.
- **Code-block language tags.** The CommonMark info string for fences, the
  `language-<x>` class that CommonMark renderers and highlight.js emit, the
  GitHub `highlight-source-<x>` class, and shebang and extension detection as
  in GitHub Linguist. All four are seed data here.
- **Provenance and re-fetch.** W3C PROV and the WARC/Memento practice of
  storing what was fetched, when, and with which digest; the `rediscovery`
  node records the same facts plus the query and rank that found the page.
- **Trust.** Wikidata's "official website" property (P856) as the anchor for
  first-party pages, as the issue specifies.

## The fix on this branch

- `rust/src/web_formalize.rs`: `formalize_page`,
  `formalize_page_with_context`, mime resolution and sniffing, HTML, Markdown,
  JSON and plain-text walkers, language-tag resolution, `url_domain`.
- `rust/src/web_formalize_trust.rs` (included): `trust_score`,
  `annotate_capture`, `network_statements`, `covers_statements`,
  `generic_page_statements`, `FormalizedPage`, `FormalizedPageStore` with
  `code_blocks_on` and `command_mentioning`.
- `data/seed/page-formalization-rules.lino` and
  `data/seed/source-trust-weights.lino`, mirrored byte-for-byte under
  `rust/embedded/data/seed/`.
- `rust/src/source_research.rs`: `formalized_page_statement` records in the
  learning proposal.

## Verification

- Offline, in CI: `rust/tests/unit/issue_1163_web_formalize.rs` (nine tests;
  every byte comes from committed fixtures through a transport that refuses
  unknown URLs). `html_page_formalizes_structure_and_code` runs against a
  fixture wrapped in `<html>` with a leading comment, so it fails if either
  the walker or the seed-wrapper fix regresses.
- The #1164 tests read code blocks out of five more captured pages through the
  same formalizer (`rust/tests/unit/issue_1164_code_example_knowledge.rs`).
- Not yet verified: a live fetch (R1163-11), routing from a real need
  (R1163-4), and any JS/TS parity (R1163-10).
