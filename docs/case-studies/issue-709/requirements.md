# Issue #709 Requirements

The issue asks for one production search operation, not another result-list
renderer. The table maps each deliverable to executable evidence. The same
cached fixture enters the public Rust operation used by CLI/HTTP/Telegram, and
the browser fixture exercises the JavaScript worker mirror.

| ID | Requirement | Evidence | Regression test |
| --- | --- | --- | --- |
| R709-01 | Formalize every search hit and fetched page as a statement with reversible source provenance. | `src/search_fusion.rs` (`FormalizedSearchObservation`, `record`, `learning_proposal`) | `cached_sources_are_formalized_merged_ranked_and_replayed_deterministically` |
| R709-02 | Merge equivalent meaning links across sources and languages; rank original sources above independent corroboration and exclude unoriginal reposts from evidence. | `src/search_fusion.rs`; `src/summarization/dedup.rs` semantic signatures; `relative_meta_logic::SourceTier` | `cached_sources_are_formalized_merged_ranked_and_replayed_deterministically` |
| R709-03 | Select the smallest useful ranked answer and retain both sides, tiers, and posteriors when sources disagree. | `src/search_fusion.rs::select_statements`; `conflict:source_disagreement` trace links | `contradictory_sources_keep_both_sides_with_tiers_and_posteriors` |
| R709-04 | Deformalize a decisive fact found only in a foreign-language source into the query language while preserving its original quote. | `src/search_fusion.rs::formalize_observation`; multilingual meaning anchors | `decisive_foreign_language_fact_is_deformalized_in_the_query_language` |
| R709-05 | Normalize URL, title, supporting quote, and localized read-more link for every presented source. | `NormalizedSearchSource`; `SearchFusionExecution::render_markdown` | `presentation_normalizes_source_url_title_quote_and_read_more` |
| R709-06 | Use the same ranked source contract on CLI, HTTP, and Telegram, including safe Telegram links and blockquotes. | `src/solver_handlers/web_requests/live_search.rs`; `src/telegram.rs` | `cli_http_and_telegram_use_the_same_ranked_source_contract` |
| R709-07 | Render the same statement/source-card model in the browser, including cross-language and conflict fixtures. | `src/web/worker/formal_ai_worker_22.js`; `tests/e2e/tests/issue-709.spec.js`; final screenshot | `web_and_telegram_acceptance_evidence_is_committed` |
| R709-08 | Replay from exact cached captures deterministically with no network access; keep live traffic behind the existing opt-in gate. | `CachedSourceClient`; `try_web_search_with_offline`; fixture request counter | `cached_sources_are_formalized_merged_ranked_and_replayed_deterministically` |
| R709-09 | Preserve raw issue/PR research, a worked formalization probe, release metadata, and one honestly attributed same-task Agent-CLI leaf. | case-study `raw-data/`; `examples/issue_709_formalization_probe.rs`; `self-hosting-authorship/`; changelog fragment | `case_study_release_and_agent_authorship_evidence_are_committed` |

## Acceptance boundary

The unit fixture captures three ranked sources and all three pages, proves the
merge and tier policy, then constructs an offline client over the warm cache.
It compares rendered Markdown, the formalize/merge/rank trace, and the complete
learning proposal byte-for-byte. The Playwright fixture intercepts every search
provider, so CI remains deterministic and network-free.
