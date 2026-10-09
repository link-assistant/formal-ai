# Next finite batch: five curated project parity failures

Repository source is held read-only until the coordinator's next integration push. This plan changes no expected native behavior and is not an implementation claim.

## Existing failing native assertions

- rust/tests/unit/specification/project_lookups.rs:37 russian_hive_mind_prompt_prefers_link_assistant_project: exact Russian promoted-body string, project:promoted evidence.
- same file:133 curated_project_concept_prompt_routes_to_project_lookup: exact link-cli promoted body, canonical repo, promoted evidence.
- same file:182 linksplatform_repository_is_promoted_by_default: exact Documentation promoted body and source repo evidence.
- same file:205 explicit_formal_ai_repository_url_still_routes_to_project_lookup: explicit repo URL stays project_lookup, exact promoted body, promoted evidence.
- same file:452 http_fetch_of_curated_github_url_describes_project_via_summarization: exact Standard summary with three highest-ranked Hive Mind statements, http_fetch:curated_project and summarization evidence.

These are already carried by scripts/lib/rust-specification-cases.mjs. Tests must use the real existing expected checks; no replacement expectations or route-only reduction.

## Shared missing mechanisms

1. Mode-aware curated descriptions: current worker describeProjectRecord drops install/example and always takes3, instead of native SummarizationConfig Short20% / Standard50% ratios. Existing generated crate/summarization.mjs supplies statementFromSeed, defaultConfig, withMode, withLanguage, summarize and deformalize. Reuse those functions on projectStatementsFor and preserve language selection; add no project-specific names, prompts, descriptions or ASTs.
2. Canonical promoted-project narrative: native render_project_lookup includes a planned web-search explanation with configured provider IDs and reciprocal rank fusion policy. Worker renderPromotedProjectLookup substitutes live/no-result prose. Move the existing native template into a seed-backed multilingual response, rendering metadata slots in both roots. When search has grounded fused results, keep actual live results/evidence/diagnostics visible. No-result/offline output must remain the canonical seeded body; do not invent successful fetch/result evidence.
3. Curated fetch projection: native curated_project_fetch matches local GitHub project records before any network access and renders existing seeded http_fetch_curated_project using Standard summaries. Worker tryFetch lacks this stage. Add the same local registry stage before fetch availability or network work, retaining explicit HTTP intent and honest curated-summary provenance. Uncurated URLs retain existing real fetch behavior.

## Finite source touchpoints and ownership to confirm after push

- js/worker/formal_ai_worker_web_search_and_projects.js: shrink existing description/render helpers; no increase to its1285-line ceiling.
- js/worker/formal_ai_worker_web_providers_and_wasm_calls.js: small curated-fetch hook; preserve its1168 ceiling by moving cohesive project/fetch projection into one new meaningful worker shard if needed.
- New meaningful worker project summary/projection shard and exact measured line-budget shard. Coordinator owns production inventory and TS final projection.
- data/seed multilingual canonical promoted lookup template (hyphenated new IDs); prefer an existing response file with available room, otherwise register a cohesive project response file through reviewed existing seed generators. Seed mirror/inventory owned by coordinator.
- rust/src/solver_handlers/web_requests.rs: render existing promoted templates from the same seed; expected final body bytes/evidence unchanged. Existing native curated_project_fetch.rs and summarization/mod.rs already implement the required Standard pipeline, so do not change them absent a new observed defect.
- New closest JS tests consume actual source-backed five cases and check every carried assertion. Existing specification-parity-worker.test.mjs route assertions remain unchanged.

## Acceptance checks

- All five original carried native cases pass in actual WorkerHost with their existing exact content/intent/evidence checks.
- A second arbitrary project record verifies Short vs Standard selection from weighted statements; declaration/name literals cannot supply missing summaries.
- Curated HTTP works with fetch unavailable and never claims a network success; uncurated HTTP still calls the real configured fetch adapter.
- A grounded live-search fixture remains visible and carries its actual evidence/diagnostics; empty or failed search adds no fabricated result.
- Localized selection (English/Russian plus native existing Hindi/Chinese behavior and fallback) is generic. Existing Spanish promoted wrapper currently falls back to English natively, so preserve that existing behavior during this narrow parity batch unless separately authorized to change it.
- Existing five native pins unchanged; source gate and new worker line budgets pass, hardcoded-language debt falls instead of growing. Native execution remains CI-only.
- Exact broad implementation ask first, record actual failure if any, then reviewed general FA capability repair blocks and original unchanged retry; no manual requirement changes. G112 remains open until broad implementation actually succeeds.

## Reserved general project weight parser parity

Coordinator reserved js/seed_loader.js project statement weight parsing for this batch after the push. Native ProjectStatement::parse trims and parses u8, falling back to50 only when parsing fails. Its legitimate range is0..255 despite the registry prose's0..100 convention. JS currently uses parseInt(...)||50, so0 is replaced, partially numeric strings are accepted and negative/out-of-range values leak into ranking.

The general parser must preserve0, accept the native optional plus sign and decimal digits after surrounding whitespace, and default50 for missing/empty/non-numeric/partial/negative/>255 values. Pin arbitrary records with0,255,+7,7x,-1,256 and missing values, using the native semantics; no known project name is needed. Use the existing shared summary comparator/percent policy unchanged. This adds one narrowly reserved source path to the finite batch and provides a meaningful mechanism pin beyond the five current positive-weight fixtures.

## Additional source-backed boundary findings

- The native description is empty for an empty statement list; it does not fall back to project.description or projectDisplayName. The new shared core adapter must preserve this edge and use default Short only for promoted descriptions, explicit Standard for curated fetch.
- projectStatementsFor has the correct native localized → English → outer fallback. Preserve it rather than copying this selection into a new parser.
- Current promoted renderer drops search.diagnostics from its return. Preserve the real grounded diagnostics along with provider/error evidence when adding the canonical narrative; a planned-provider event is not a completed-fetch event.
- Canonical promoted output includes planned-provider and fusion evidence natively. Preserve those canonical plan events even when the browser also carries actual provider attempt/result evidence.
- Existing issue-0133-search-fusion.test.mjs provides the genuine production fetch-fixture shape: site files load from disk; only provider HTTP is scripted, then actual extraction and fusion run. Use the same narrow technique for the project live-result regression instead of replacing runWebSearchQuery with fabricated success.
- Generic empty and localized statement sets, weight0 ordering, tie stability, malformed weights, and no-network curated projection all have meaningful checks beyond the five current positive-weight native cases.
