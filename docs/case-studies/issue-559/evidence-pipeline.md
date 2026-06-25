# Issue 559 Evidence Pipeline

This document specifies the general fresh-data evidence pipeline (R10, R22). It is
written to reuse what already exists — multi-provider search and Reciprocal Rank
Fusion are **built** (see [critical-review.md](critical-review.md) CR4) — and to
add only the genuinely missing stages: crawl, extract, contradiction check,
hypothesis formation, and live non-CORS providers.

The pipeline is one method the recursive core can invoke for any unit whose
evidence policy requires fresh data; it is not a new top-level branch.

## What Already Exists (reuse, do not rebuild)

- `src/web_search_core.rs` — `no_std`, network-free symbolic core:
  - `WEB_SEARCH_PROVIDER_REGISTRY` (`:90-315`), **33 providers** across
    Search / Knowledge / Papers / Code categories, including `google` (`:99`,
    `cors_readable:false`), `bing`, `brave`, `duckduckgo`, Wikipedia, Wikidata,
    Wiktionary, arXiv, GitHub.
  - `WEB_SEARCH_PROVIDERS` (`:327-334`) — the 6 CORS-readable ids:
    `duckduckgo, internet-archive, wikipedia, wikidata, wiktionary, wikinews`.
  - `reciprocal_rank_fusion(entries, k)` (`:396-438`), `WEB_SEARCH_RRF_K=60`
    (`:33`).
- `src/solver_handlers/web_requests.rs` — `try_web_search` (`:155-276`),
  `try_http_fetch` (`:22-85`), `try_url_navigate` (`:116-140`). These are
  **descriptive**: they emit the plan (e.g. `web_search:combined rrf:k=60`) and
  return prose; they perform no network I/O.
- `src/web/formal_ai_worker.js` — the real engine: live provider dispatch and
  concurrency (~`:35736`–`:36254`), `reciprocalRankFusion` (~`:35880`) calling
  the WASM export `wasm.web_search_fuse`, and `tryFetch` (~`:34849`–`:34937`),
  a real CORS GET truncated to 2000 bytes.
- `desktop/lib/tool-router.cjs:92-115` — a permission-gated `http_fetch` plain GET
  (the non-CORS seam).
- Provenance fields already standardized (R67): `source:http`, `fetched_at`,
  `sha256`, `cache_hit`, `policy:offline`, backed by `data/cache/`.

## What Is Absent (build this)

Verified absent (`grep -rni crawl` over `src/`, `desktop/`, `vscode/` → 0 hits):

- crawling / full-content extraction of reranked result pages;
- live non-CORS providers (Google/Bing/Brave are registered but
  `cors_readable:false`);
- a general expand → search → rerank → crawl → extract → compare → hypothesize
  loop reusable by any method;
- contradiction detection across sources;
- hypothesis formation that feeds gaps back into the recursive core.

## Pipeline Stages

The pipeline is defined once as a `.lino` evidence recipe + Rust policy
(canonical), and executed by whichever runtime is available (browser worker for
CORS providers, desktop fetch seam for non-CORS). Stages:

### 1. Expand

Turn the unit's question into a set of queries: terms, phrases, full sentences,
and explicit questions (R22 "search each term, phrase, sentence, and question").
Expansion uses the formalized frame's knowns/relevants (no new hardcoded English
cue lists — those move to seed data per R97). Output: a bounded query set.

### 2. Search (reuse)

Run the query set across providers using the existing registry. CORS-readable
providers (`WEB_SEARCH_PROVIDERS`) run in the browser worker; non-CORS providers
run through the desktop fetch seam (Decision 4B). Offline mode
(`FORMAL_AI_OFFLINE`) short-circuits to `data/cache/`.

### 3. Rerank (reuse)

Fuse per-provider rankings with the existing `reciprocal_rank_fusion`
(`WEB_SEARCH_RRF_K=60`). No change to the algorithm; the new work is feeding more
providers into it.

### 4. Crawl (new)

Fetch the top-N reranked result pages. The browser path extends `tryFetch`; the
desktop path uses `http_fetch`. Each fetch records full provenance (R67) and is
cache-first. The 2000-byte truncation in the current `tryFetch` is lifted for
crawl (with a size cap and sharding to respect the `.lino` 1500-line cap — at
which point `links-notation#197` streaming becomes relevant, per the upstream
audit).

### 5. Extract (new)

Pull candidate facts/spans from crawled content, each linked back to its source
(doublet links, `VISION.md:44`). Extraction is conservative and records the
source span so every extracted claim is attributable (matching the
`meta-language` source-span direction, R24).

### 6. Compare / contradiction check (new)

Cluster extracted claims by the question they answer; flag agreements and
contradictions across sources. Contradictions are recorded as links between the
conflicting claims, not silently resolved.

### 7. Hypothesize (new)

Form a best-supported answer with a confidence note, and emit any unresolved
gaps as new needs back into the recursive core
([recursive-core.md](recursive-core.md)). This closes the loop: missing evidence
becomes a new `WorkUnit`, not a dead end.

## Execution Topology (Decision 4)

| Runtime | Providers it can reach | Crawl seam | Notes |
| --- | --- | --- | --- |
| Browser worker | 6 CORS-readable (`WEB_SEARCH_PROVIDERS`) | `tryFetch` (extend) | The real search engine today |
| Desktop (Electron) | non-CORS (Google/Bing/Brave) + crawl | `http_fetch` (`tool-router.cjs:92`) | Permission-gated; unlocks `cors_readable:false` |
| Rust (canonical) | none live today (descriptive) | — | Defines the pipeline as data/policy; future Rust HTTP client is Decision 4C |
| Offline | cache only | — | `FORMAL_AI_OFFLINE` → `data/cache/`; deterministic |

The Rust layer stays canonical for *policy and shape* even though it does not
perform live I/O today, matching `ARCHITECTURE.md` §10.2 ("the Rust pipeline is
the canonical implementation"). Execution is delegated, parity is preserved by
keeping the policy in one place.

## Evidence Policy (R67, extends existing)

The evidence policy is a `SolverConfig`-level extension of the existing
source/offline policy (added to config first, per `NON-GOALS.md`). It decides per
unit whether cached evidence suffices or fresh data is required, based on the
need type (time-sensitive, factual, recommendation-like — R10). It never bypasses
offline mode and always records provenance, so behavior stays deterministic and
auditable.

## Parity And Determinism Constraints

- Any logic shared between Rust and the worker (e.g. RRF, query expansion rules)
  must keep byte-for-byte parity (`ARCHITECTURE.md` §10.2, §895-896) and gain a
  wired parity check — addressing the weak-flank risk in
  [critical-review.md](critical-review.md) (most `experiments/*-parity.mjs` are
  not in CI today).
- CI tests run against **cached fixtures**, never live network, so the suite stays
  deterministic and offline-capable.
- The pipeline respects total reference closure: extracted/crawled `.lino` must be
  authored closed or excluded like `data/cache/wikidata/` already is in
  `scripts/check-file-size.rs`.

## Comparison Harness

Per Decision 4 in [options-comparison.md](options-comparison.md): an
evidence-quality benchmark scores answers with the pipeline **off** vs **on**, and
**CORS-only** vs **CORS+desktop**, on a fixed question set with cached fixtures.
This quantifies the marginal value of crawl and non-CORS providers before either
becomes default-on, so "implement them all to compare" is realized as a measured
benchmark rather than an assertion.

## Tests

- Unit tests for each new stage (expand, crawl, extract, compare, hypothesize)
  against fixtures.
- A grounding entry in the general meta recipe describing the pipeline, asserted
  against live source (resolves C5).
- Prompt-variation cases (R129) for fresh-data questions across languages.
- A parity check for any shared expansion/fusion logic.
