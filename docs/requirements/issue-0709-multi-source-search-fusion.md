## Issue #709 Multi-Source Search Fusion

Issue [#709](https://github.com/link-assistant/formal-ai/issues/709) composes
exact search/page capture, multilingual formalization, #844's statement merge,
relative source tiers, and normalized presentation into one ranked answer. See
`docs/case-studies/issue-709/` and PR #884.

| ID | Requirement | Status |
| --- | --- | --- |
| R709-1 | Formalize every captured search hit and fetched page with source provenance. | Delivered: `execute_search_fusion` records a `FormalizedSearchObservation` plus event-log and learning-proposal receipts for each statement. Pinned by `rust/tests/unit/issue_709_search_fusion.rs` (`cached_sources_are_formalized_merged_ranked_and_replayed_deterministically`, `browser_wasm_core_deformalizes_and_preserves_exact_provenance`). |
| R709-2 | Merge equivalent meanings across languages and rank them using original, independent, and unoriginal source tiers. | Complete Q/P/Q meaning links enter #844's semantic signature; reposts are traced but excluded from evidence. Pinned by `rust/tests/unit/issue_709_search_fusion.rs` (`cached_sources_are_formalized_merged_ranked_and_replayed_deterministically`, `capture_policy_is_per_statement_and_demotes_exact_mirrors`). |
| R709-3 | Deformalize the smallest sufficient ranked answer into the query language and show both conflict sides with posteriors. | Delivered: The selection is bounded to three meanings, retains both polarities, and emits `conflict:source_disagreement`. Pinned by `rust/tests/unit/issue_709_search_fusion.rs` (`decisive_foreign_language_fact_is_deformalized_in_the_query_language`, `contradictory_sources_keep_both_sides_with_tiers_and_posteriors`, `browser_wasm_core_keeps_both_ranked_conflict_sides`). |
| R709-4 | Normalize URL, title, quote, and read-more fields across web, CLI/HTTP, and Telegram. | `NormalizedSearchSource`, the shared Rust Markdown renderer, Telegram HTML conversion, and the browser worker source cards are covered by unit and Playwright fixtures: `rust/tests/unit/issue_709_search_fusion.rs` (`presentation_normalizes_source_url_title_quote_and_read_more`, `cli_http_and_telegram_use_the_same_ranked_source_contract`) and `rust/tests/e2e/tests/issue-709.spec.js`. |
| R709-5 | Replay deterministically in CI while live search remains explicitly gated. | Delivered: A three-source exact-capture fixture compares the live and offline render, trace, and proposal byte-for-byte; browser providers are intercepted. Pinned by `rust/tests/unit/issue_709_search_fusion.rs` (`cached_sources_are_formalized_merged_ranked_and_replayed_deterministically` replays offline and compares render, trace and proposal). |
