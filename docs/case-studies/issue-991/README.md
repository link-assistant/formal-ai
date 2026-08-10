# Issue 991 case study — a discovery plan is not a guide

**Issue:** [#991 — Complete dynamic multi-source how-to synthesis and seven-day service caching](https://github.com/link-assistant/formal-ai/issues/991)
**Parent:** [#710](https://github.com/link-assistant/formal-ai/issues/710) row 20, the remainder after [#444](https://github.com/link-assistant/formal-ai/issues/444)/[PR #448](https://github.com/link-assistant/formal-ai/pull/448) and [#709](https://github.com/link-assistant/formal-ai/issues/709)/[PR #884](https://github.com/link-assistant/formal-ai/pull/884)
**Closing pull request:** <https://github.com/link-assistant/formal-ai/pull/995>

## The reported symptom

Asked `how to make pancakes?`, every surface described what it *would* do
(`raw-data/before-fix-run.txt`, captured from a server built at this branch's
base commit):

```text
Procedural discovery plan for `make pancakes` (action `make`, object `pancakes`).

I do not answer this from a memoized recipe. The solver first checks Wikipedia for
topic context and Wikidata for entity/action/object hints. It then tries wikiHow's
CORS-readable MediaWiki parse API candidate `Make-Pancakes` ...
```

The plan was truthful and it was not a guide. Nothing executed the registered
services, nothing merged their steps, and nothing carried provenance, because
the synthesis step the plan describes did not exist. Three further gaps came
with it: search results were never followed to the page that actually holds the
procedure, service reachability was inferred from a body cache that only records
successes, and no committed capture let the suite prove any of this offline.

## Root causes

**RC1 — the plan was the terminal state.** `src/solver_handler_how.rs` composed
the discovery plan and returned it. There was no code path from "these services
are registered and enabled" to "these are the steps, and here is where each one
came from", so no amount of network access could have produced a guide.

**RC2 — the first hop is rarely the procedure.** Stack Exchange's
`search/advanced` returns *questions*; a question body has no ordered list in
it. The steps live one hop deeper, in `/questions/{id}/answers`. Without
recursion inside a declared bound, a technical task returns nothing at all even
though the service answers it well.

**RC3 — a body cache is not an availability record.** `CachedSourceClient`
stored successful response bodies for 60 days. A service that returned 500, or
that could not be reached at all, left no trace, so the next request paid the
same timeout again and the environment never learned anything about the service.

**RC4 — nothing offline could fail.** With no committed captures, any regression
would have had to reach the live network — which makes it non-deterministic in
CI — or stub the transport, which proves nothing about the production path.

## The fix

**One bounded synthesis contract, executed by both runtimes.**
`src/how_to_guide.rs` (with `extract.rs` and `render.rs`) selects sources from
`data/seed/sources-registry.lino` — never a hardcoded service list — walks each
one inside `GuideBounds { max_depth: 2, max_pages_per_service: 4, max_services:
4, max_steps: 12, max_capture_age_seconds: 5_184_000 }`, and orders the accepted
steps by the #709 source tier, then depth, then source, then position. Every
step keeps its source id, exact URL, license, capture digest, and depth.
`src/web/worker/formal_ai_worker_24.js` mirrors that contract for the browser.

**Recursion to the page that holds the procedure.** A Stack Exchange question
judged relevant at depth 0 is followed to its answers at depth 1; a wiki page
with no list items is followed to the same-wiki articles it links. Relevance is
judged only at depth 0 — deeper captures belong to a page already accepted — and
the bounds cap the walk.

**A seven-day accessibility memory.** `src/service_accessibility.rs` records
success *and* failure per service with a TTL of `7 * 24 * 60 * 60` seconds in
the environment's associative memory, with explicit `needs_refresh`,
`invalidate`, and `invalidate_all`. A failure on a service's declared entry
endpoint marks it unreachable; a failure on a fallback endpoint is recorded as
`fallback_failed`, so one broken search API does not poison a service that
answers its parse API fine.

**Committed captures, offline replay, gated drift check.**
`tests/fixtures/issue-991/` holds the real wikiHow, Stack Exchange, Rosetta
Code, and Wikibooks responses with their timestamps, sha256 digests, byte
counts, and licenses in `capture-manifest.lino`. The normal suite replays them
with the transport disabled. `FORMAL_AI_LIVE_FETCH=1` re-fetches through the
same production path and reports any drift.

## Evidence

`raw-data/before-fix-run.txt` is the same request answered by a binary built at
the base commit: the discovery plan, no steps, no citations, no digests.

`raw-data/after-fix-run.txt` records, in one file:

1. a real `formal-ai serve` process answering `how to make pancakes?` over
   `/api/openai/v1/chat/completions` with twelve steps, each citing
   `wikiHow (CC BY-NC-SA 3.0, sha256 1cbdf5f7a9d6)`, a `### Sources` section
   naming every service consulted and what it returned, and the bounds the run
   used;
2. the same server answering `how to build a nonexistent quantum flux
   capacitor?` — no service documents it, so no procedure is asserted and the
   discovery plan remains the answer;
3. the browser worker producing the identical guide from the identical bytes.

`raw-data/github/` preserves the issue, its comments, and the pull request as
fetched.

## Tests

| Surface | File | What a failure means |
| --- | --- | --- |
| Native | `tests/unit/issue_991_how_to_synthesis.rs` | The synthesis contract, the opt-out authority, the depth bound, the seven-day TTL, the digest replay, the cross-runtime parity, or the drift check broke. |
| HTTP | `tests/integration/issue_991_how_to_http.rs` | A real server process no longer answers a how-to request from the committed captures with provenance. |
| Browser | `tests/web/issue-991-how-to-synthesis.test.mjs` | The worker mirror drifted from the Rust behaviour or from `sources-registry.lino`. |

The native and browser suites both assert against
`tests/fixtures/issue-991/expected-guides.json`, written from the Rust
production path by `examples/issue_991_how_to_parity.rs`. Two runtimes checked
against one recorded expectation cannot drift apart without a test failing.

## Known limitation

Relevance is lexical: `matches_task` accepts a Stack Overflow question whose
title literally names the topic, so a well-titled but weakly-answered question
can still supply steps. That is a ranking question, not a provenance one — every
step it admits still carries the exact URL and digest a reader can check — and
the bounds keep the cost fixed. Tightening it belongs with the source-ranking
work in #709 rather than here.
