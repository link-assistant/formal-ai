Parent: #651

## Motivation and evidence

A repeatedly-stated pipeline was never delivered end to end: **translate all search results into the single meta language, merge them, and present the most relevant statements by deformalization**:

- Issue #505: exactly this ask — only web-search routing shipped.
- Issue #444: dynamic multi-source synthesis of how-to answers — not evidenced in PR #448.
- Issue #63: merge multi-language Wikipedia definitions of the same concept (deterministic fusion) with reversible translation learning — zero response at close; only the seed-concept fusion slice exists (R-issue-63 block).
- Issue #153/#180: Google-style normalized results (url + title + quote + "Read more"), dedup/joining across engines, source priority — konard noted #153 "was already ignored", and #180's restatement was also dropped.
- Issue #481: "reconstruct reasoning to the smallest atomic links substitution operations" for a Google-quality answer — only intent routing shipped.

This is the knowledge-side generality engine: answers built by fusing formalized statements from several sources, ranked by the symbolic probability layer, with every statement carrying provenance.

## Requirements

1. **Formalize every result**: each search hit / fetched source is converted into meta-language statements (reusing the formalization pipeline and the #563 resource-formalization boundary) with `source:` provenance per statement — not kept as opaque text snippets.
2. **Merge and dedupe by meaning**: statements from different sources/languages that formalize to the same meaning links are merged (one statement, N sources), increasing the statement's evidence weight under the relative-meta-logic tiers (original first sources highest, reposts ignored — the #535 policy).
3. **Rank and select**: the answer presents the most relevant fused statements by deformalization into the user's language, smallest-sufficient, with per-statement source chips; conflicting statements surface as recorded disagreements (`conflict:source_disagreement`) with both sides shown.
4. **Cross-language fusion**: a query answered poorly by sources in the user's language must exploit sources in other languages through the meta language (the #63 requirement) — demonstrated by a fixture where the decisive statement exists only in a non-query-language source.
5. **Normalized presentation**: the classic #153 asks — url + title + quoted supporting fragment + "read more" per source, dedup across engines — on web, CLI, and Telegram surfaces.
6. Determinism/offline: recorded search/fetch fixtures make the whole pipeline replayable in CI; live mode stays behind the existing `FORMAL_AI_LIVE_API` gates.

## Acceptance criteria

- An open question with ≥ 3 cached sources yields an answer whose trace shows per-statement formalization, a merged statement with multi-source evidence, and ranked selection; repeated runs identical.
- The cross-language fixture (requirement 4) passes: the answer cites the foreign-language source and renders in the query language.
- A contradiction fixture shows both sides with tiers and posteriors instead of picking one silently.
- Presentation e2e on web + Telegram matches the normalized format.

## Dependencies

- Blocked by #661 (E42 statement weights) — the ranking substrate.
- Related: #660 (E41), world-models (contexts hold the fused statements), #687 (web search/fetch firing), closed parents #505/#444/#63/#153.

## Process

Collect data to `docs/case-studies/issue-{id}` (fusion design, RRF and source-tier research already partly in issue-444/issue-535 studies, survey: RDF named graphs provenance, TruthDiscovery literature — symbolic methods only); single PR until every requirement is addressed.

