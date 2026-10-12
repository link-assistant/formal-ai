# Issue #1173 Case Study: The Fallback That Described a Search Instead of Running It

Issue [#1173](https://github.com/link-assistant/formal-ai/issues/1173) (E138,
part of the #1183 umbrella). Verified on `main` at `d209aac64`; fixed on the
`qa-reasoning-coding-bulk-fixes` branch.

## What a user saw

Any prompt no handler claimed — an unresolved instruction like "Calibrate the
snorflax against silent teal weather", or a research-shaped question the
intent recogniser did not map — ended in this final answer:

> Web search requested for `calibrate the snorflax against silent teal
> weather`.
>
> In the browser demo formal-ai defaults to the DuckDuckGo Instant Answer
> endpoint (CORS-readable, keyless) and queries Internet Archive, Wikipedia
> REST, Wikidata, Wiktionary, and Wikinews in that priority order. …
>
> Provider: duckduckgo (default)
> Providers considered: …
> Combined ranking: reciprocal rank fusion (k = 60)

No search ran. No page was fetched. Nothing was answered. The paragraph
described what a *browser demo* would do, in every language the engine
supports (`Поиск в интернете запрошен для …` in Russian), for every class
without its own handler — rewriting, classification, extraction, SQL,
arithmetic — which is 22 of the 34 parity probes of #1171 (E136).

## Root cause

Two code paths met in the wrong place:

1. **The paragraph**: `answer_web_search_query`
   (`rust/src/solver_handlers/web_requests.rs`) had six `format!` arms —
   default, latest-news, open-research, each in en and ru — and every one of
   them only *described* the search machinery. It finalised with confidence
   `0.8` and intent `web_search`, so the description was presented as a
   real answer.

2. **The fallback**: when no handler claims a prompt,
   `rust/src/solver_unknown_reasoning.rs` picks a focus phrase and handed it
   to that same function with `WebSearchQueryKind::UnknownReasoningFallback`
   (`solver_unknown_reasoning.rs:123`). So every class without its own
   handler ended in a description of a search that never ran.

3. **The executed search already existed**:
   `try_web_search_with_client` → `crate::search_fusion::execute_search_fusion`
   (`rust/src/solver_handlers/web_requests/live_search.rs`) captures provider
   responses, formalises pages into statements, fuses them with reciprocal
   rank fusion, renders citations, and degrades to a localized
   `web_search_unavailable` response when offline. It was only reached from
   `solver_dispatch.rs:209`, for prompts whose intent is *explicitly* a
   search — exactly the prompts that did not need the help.

## The change

| File | Change |
| --- | --- |
| `rust/src/solver_handlers/web_requests/live_search.rs` | New `execute_web_search_answer(prompt, query, kind, log, client)` — the shared executed core: logs `web_search:request` / `web_search:query_kind`, runs the fusion, records `web_search:executed` with the query and result counts, answers from the rendered statements, or degrades to the localized `web_search_unavailable` response (recording the missed capture as `error:fetch`, and `policy:offline` for direct page-fetch misses). New `live_fetch_enabled()` reads the `FORMAL_AI_LIVE_FETCH` opt-in. `try_web_search_with_client` now delegates to the core. |
| `rust/src/solver_handlers/web_requests.rs` | All six descriptive `format!` arms of `answer_web_search_query` are **deleted**. The function keeps its name (it is re-exported by `solver_handlers::mod`) but now builds the process-cache client — online only when the caller's runtime is not offline **and** `FORMAL_AI_LIVE_FETCH` opted in, the same combined gate `solver_dispatch.rs` applies — and executes through the shared core. `try_web_search` delegates to it. |
| `rust/src/solver_unknown_reasoning.rs` | The fallback arm keeps its gating (consult-walk precedence, bare-term escape, offline boundary, unsupported languages) and its `reasoning:candidate_source` / `reasoning:gather_attempt` events, and now calls the executed `answer_web_search_query(prompt, focus, kind, log, config.offline)`. |
| `rust/tests/unit/issue_1173_fallback_executes_search.rs` | New acceptance suite (below). |

## Before and after

Representative prompts; the "before" bodies are the deleted arms (quoted from
`main`), the "after" bodies follow from the localized seed templates and the
issue #709 renderer. The full 22-probe grid of #1171 is re-run by that
issue's lane; what this case study pins is the fallback's own rule.

### The unresolved instruction (offline, live fetch off — the default)

Prompt: `Calibrate the snorflax against silent teal weather`

- **Before**: the `Web search requested for …` paragraph above. Intent
  `web_search`, confidence `0.8`, nothing executed.
- **After**: the executed search runs against the process source cache; with
  no capture for the focus it misses and the answer is the localized
  template (en):
  > No captured provider response is available for `…`. Live fetching is off
  > by default; enable it explicitly to populate the replayable source cache.
  > The unexecuted plan includes DuckDuckGo, Internet Archive, Wikipedia,
  > Wikidata, Wiktionary, and Wikinews.
  Intent `web_search`, confidence `0.0`, `policy:offline` recorded. With
  `FORMAL_AI_LIVE_FETCH=1` (or a captured response in the cache) the same
  path answers from the fused statements with citations instead.

### The explicit search (`Search the web for …`)

- **Before**: the same paragraph, via the `try_web_search` native handler.
- **After**: identical executed path — captured sources answer with
  per-statement citations (`posterior`, source tier, URL, quote, read-more
  link); offline with no capture, the `web_search_unavailable` response.

## Why the offline boundary still holds

`answer_web_search_query` applies `runtime_offline || !live_fetch_enabled()`
before constructing the `CachedSourceClient`. An offline runtime therefore
never constructs an online client, offline cache reads still happen (valid
captures answer), and the issue #873 no-network boundary test
(`offline_mode_preserves_the_explicit_no_network_boundary`) is untouched: the
fallback arm's own gate still refuses to search at all for non-bare-term
prompts when the runtime is offline.

## Tests

`rust/tests/unit/issue_1173_fallback_executes_search.rs`, all hermetic (a
fixture `SourceTransport`; the fallback case runs with the transport off):

1. `executed_search_answers_from_captured_sources_with_citations` — the
   executed answer names the query, cites the read pages, rests on captured
   statements, and records `web_search:executed:query=…`.
2. `offline_cache_miss_answers_unavailable_not_a_description` — offline with
   an empty cache degrades to `web_search_unavailable` naming the query,
   with `policy:offline` recorded.
3. `unknown_reasoning_fallback_executes_instead_of_describing` — the
   canonical #873 unresolved instruction reaches the fallback through
   `UniversalSolver::solve` and yields the unavailable response (the executed
   attempt's honest miss), never the paragraph.

Every test also asserts the canned openers (`Web search requested`,
`Поиск в интернете запрошен`, `Providers considered`) appear in no answer
body. Command: `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit
issue_1173_fallback_executes_search`.

## Honest boundaries

- Page-content hashes in the executed answer are #1184's derivation and are
  not delivered here; the citations carry URL, title, and quote today.
- Routing classes away from the fallback (#1175) and browser-worker parity
  (R5) are sibling lanes of this pull request.
- The `rust/tests/source/` snapshot of `web_requests.rs` still carries the
  old paragraph; that tree is a stale compiled snapshot that has drifted
  from `rust/src/` since the three-roots layout commit (`70df98cf6`) and has
  no sync gate — bringing it back in sync is its own task.
