## Parent requirement

This tracks the portion of [#710](https://github.com/link-assistant/formal-ai/issues/710) row 20 that remains after [#444](https://github.com/link-assistant/formal-ai/issues/444), [PR #448](https://github.com/link-assistant/formal-ai/pull/448), [#709](https://github.com/link-assistant/formal-ai/issues/709), and [PR #884](https://github.com/link-assistant/formal-ai/pull/884).

PR #448 delivered procedural follow-up rebinding, the external-service registry/settings, benchmark metadata, and a discovery-plan shape. PR #884 delivered statement-level multi-source search fusion. Those are useful prerequisites, but they do not close the original production contract.

## Current reproduction

The current issue-444 case study explicitly records these requirements as unfinished under `What remains genuinely network-dependent (R4 depth, R7, R9, R10)`:

- fully reasoned cross-source step synthesis;
- recursive crawling of search results;
- per-service accessibility caching for at least seven days;
- QA captures from real services with fast cached replay.

The native `src/solver_handler_how.rs` emits a discovery plan and request events rather than executing a source-backed synthesized guide. The browser worker can use one wikiHow result or fall back to web search, but does not construct one reasoned guide from the registered service set. `CachedSourceClient` stores successful response bodies for 60 days; it does not record a per-service success/failure accessibility state in associative memory.

## Expected

A procedural how-to request should execute the registered, enabled trusted sources, recursively capture useful result pages within explicit bounds, and synthesize one ordered guide whose individual steps retain source provenance. Each environment should remember service accessibility status, including failures, for at least seven days without treating a stale body cache as an availability record.

## Acceptance criteria

- Rust/server and browser-worker production paths execute the same bounded source-selection and guide-synthesis contract.
- Enabled relevant services from `sources-registry.lino` can contribute; settings opt-outs remain authoritative.
- Search results are recursively captured within declared depth/page/time bounds and every accepted step retains exact provenance.
- Conflicts, copied sources, and insufficient evidence are handled using the #709 source-tier and contradiction policy.
- Per-service accessibility success/failure status is stored in the environment associative memory with a TTL of at least seven days and has explicit refresh/invalidation behavior.
- Real-service QA captures are committed with timestamps, hashes, and licenses/provenance; the normal test suite replays them offline and a gated refresh check detects drift.
- Minimal native, HTTP agent/API, and browser regressions fail before the implementation and pass through the real production paths afterward.
- The #710 row-20 audit remains `still-broken` until every item above is demonstrated.
