# Issue #709 Requirements

The issue asks for one production search operation, not another result-list
renderer. The table maps each deliverable to executable evidence. The same
cached fixture enters the public Rust operation used by CLI/HTTP/Telegram, and
the browser fixture exercises the Rust/WASM core through its JavaScript bridge.

| ID | Requirement | Evidence | Regression test |
| --- | --- | --- | --- |
| R709-01 | Formalize every search hit and fetched page as a statement with reversible source provenance. | `src/search_fusion.rs` (`FormalizedSearchObservation`, `record`, `learning_proposal`) | `cached_sources_are_formalized_merged_ranked_and_replayed_deterministically` |
| R709-02 | Merge equivalent meaning links across sources and languages; rank original sources above independent corroboration and exclude unoriginal reposts and exact mirrors from evidence. | `src/search_fusion.rs`; Agent-authored `data/meta/search-fusion-source-policy.lino`; `relative_meta_logic::SourceTier` | `cached_sources_are_formalized_merged_ranked_and_replayed_deterministically`, `capture_policy_is_per_statement_and_demotes_exact_mirrors` |
| R709-03 | Select the smallest useful ranked answer and retain both sides, tiers, and posteriors when sources disagree. | `src/search_fusion.rs::select_statements`; `conflict:source_disagreement` trace links | `contradictory_sources_keep_both_sides_with_tiers_and_posteriors` |
| R709-04 | Detect language per statement and deformalize a decisive foreign fact into the query language with data-driven target grammar while preserving its original quote. | `src/search_fusion.rs::formalize_observation`; `src/search_fusion_grammar.rs`; multilingual meaning anchors | `decisive_foreign_language_fact_is_deformalized_in_the_query_language`, `native_deformalization_uses_data_driven_hindi_word_order`, `browser_wasm_core_detects_each_statement_language_and_uses_target_grammar` |
| R709-05 | Normalize URL, title, supporting quote, and localized read-more link for every presented source. | `NormalizedSearchSource`; `SearchFusionExecution::render_markdown` | `presentation_normalizes_source_url_title_quote_and_read_more` |
| R709-06 | Use the same ranked source contract on CLI, HTTP, and Telegram, including safe Telegram links and blockquotes. | `src/solver_handlers/web_requests/live_search.rs`; `src/telegram.rs` | `cli_http_and_telegram_use_the_same_ranked_source_contract` |
| R709-07 | Render the same statement/source-card model in the browser, including cross-language and conflict fixtures. | `src/web_search_fusion_core.rs`; `src/web/worker/formal_ai_worker_23.js`; `tests/e2e/tests/issue-709.spec.js`; final screenshot | `browser_wasm_core_deformalizes_and_preserves_exact_provenance`, `browser_wasm_core_keeps_both_ranked_conflict_sides`, `web_and_telegram_acceptance_evidence_is_committed` |
| R709-08 | Replay exact cached captures deterministically; infer a reusable recipe only from distinct successful executions; require a zero-failure held-out gate and named review before durable execution. | `CachedSourceClient`; `src/search_fusion_learning.rs`; Agent-authored learning contract and held-out fixture | `cached_sources_are_formalized_merged_ranked_and_replayed_deterministically`, `execution_frontier_infers_only_after_two_independent_successes`, `one_execution_cannot_be_counted_twice_under_different_task_ids`, `candidate_is_inert_until_both_green_gate_and_named_review`, `approved_recipe_round_trips_and_executes_a_held_out_task` |
| R709-09 | Preserve raw issue/PR research, a worked probe, release metadata, honestly attributed same-task Agent-CLI leaves, and a live Formal-AI-derived learning report. | case-study `raw-data/`; `examples/issue_709_formalization_probe.rs`; `self-hosting-authorship/`; `agent-cli-evidence/`; changelog fragment | `case_study_release_and_agent_authorship_evidence_are_committed`, `associative_report_is_derived_from_agent_authored_observations`, `formal_ai_executes_the_issue_709_learning_report_recipe` |

## Acceptance boundary

The unit fixture captures three ranked sources and all three pages, proves the
merge and tier policy, then constructs an offline client over the warm cache.
It compares rendered Markdown, the formalize/merge/rank trace, and the complete
learning proposal byte-for-byte. The Playwright fixture intercepts every search
provider, so CI remains deterministic and network-free.
