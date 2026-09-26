## Issue #412 PR Review Standards (comment 4674…, knowledge-source breadth)

The 2026-06-11 review on issue #412 rejected deferring the broad
requirements and demanded they ship in the same PR as the numeric-list
coreference fix: incorporate external knowledge sources as cached APIs,
bound every cache, and generalise the coding answers beyond the built-in
catalogue. These standards govern that work.

| ID | Requirement | Status |
| --- | --- | --- |
| R289 | Public knowledge bases that expose no machine API (Rosetta Code, Wikifunctions, the Hello World Collection, Stack Overflow) must still be usable as external sources: a reviewed snippet (with its deterministic output and source attribution) is cached as a popular example and merged into the solver's answers like any other API. The coding catalogue must generalise to languages it does not template. | Implemented first by the bounded cached oracle in `rust/src/knowledge.rs` and `rust/src/solver_handler_oracle.rs`. The 2026-09-15 #710 continuation adds live, cache-backed Wikifunctions and Rosetta Code MediaWiki consumers under `rust/src/coding/function_catalog/`, with license, source hash, offline replay, and bounded execution tests in `coding_discovery::{wikifunctions,rosetta}`. |
| R290 | No cache may mirror a whole source: the local copy is capped at 1% of the source, or 512 items when 1% is smaller, per source / API / merged topic. CI must keep the committed cache under the cap. | Implemented by `cache_capacity` / `within_cache_capacity` / `KNOWLEDGE_CACHE_FLOOR` in `rust/src/knowledge.rs` (1% rounded up, floored at 512, clamped to source size) and the ratchet test `committed_snapshots_stay_within_the_cache_cap`, which fails if any per-source snapshot count exceeds the cap. |
| R291 | Every reasoning surface must agree: a fix in the Rust solver must be mirrored in the WASM browser worker so the native binary, the desktop/VS Code shells, and the web demo return byte-identical answers. | Implemented by mirroring the oracle data + lookup + renderer in `js/worker/formal_ai_worker.js` (`CODING_ORACLE_SNAPSHOTS`, `codingOracleLookup`, `codingOracleAnswer`); `experiments/issue-412-js-oracle.mjs` drives the worker's `tryWriteProgram` and the rendered answer is verified byte-identical to the Rust `solve()` output. |
| R292 | These broad requirements must ship in this PR, not be deferred: the oracle, the bounded-cache policy, and the cross-runtime mirror are all delivered here, with the popular-case cache committed as the offline accelerator a gated live refresh would repopulate. | Implemented in this PR; the live-refresh path follows the existing `FORMAL_AI_LIVE_API` discipline and the committed snapshots are the popular-case cache it repopulates. |
