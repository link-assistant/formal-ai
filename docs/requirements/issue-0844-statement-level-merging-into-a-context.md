## Issue #844 Statement-Level Merging Into A Context

Issue [#844](https://github.com/link-assistant/formal-ai/issues/844) asks for
statement-level deduplication, evidence-weighted importance, recursive source
gathering with a recheck before presenting, a merge target that is a context
rather than a list, and an identifier rung below the topic rung. PR
[#855](https://github.com/link-assistant/formal-ai/pull/855) implements all of
it through the exact-capture and named fact-checking boundaries delivered by
issues #843 and #845. `SourceProvider` remains only as a deterministic fixture
seam. See `docs/case-studies/issue-844/`.

| ID | Requirement | Status |
| --- | --- | --- |
| R501 | Deduplicate at the statement level so N sources asserting one fact yield one fact, each merge recorded as an explainable link. | Implemented by `src/summarization/dedup.rs` (`StatementSignature`, `MergedStatement`, `MergeLink`); covered by `n_sources_asserting_one_fact_yield_one_statement_with_a_justification_link`. |
| R502 | Stay conservative: merge wording differences only, never stem, so a different inflection or extra content stays a separate fact (`NON-GOALS.md:39`). | Implemented by the content-term signature over seed-known function words; covered by `wording_differences_merge_but_extra_content_does_not` and `inflected_wordings_stay_separate_because_the_merge_does_not_stem`. |
| R503 | Make every merge reversible, so a merge that conflates two facts can be split back into its variants. | Implemented by `DedupReport::split`; covered by `a_merge_that_conflates_two_facts_can_be_split`. |
| R504 | Weight importance by evidence: the kind prior blended with observed link frequency, source authority, and stance, with unoriginal mirrors adding neither probability nor ranking evidence. | Implemented by `src/summarization/importance.rs`; covered by `ranking_reflects_observed_frequency_and_source_stance`, `an_unoriginal_mirror_adds_no_probability`, and `unoriginal_repetition_cannot_outrank_an_authoritative_source`. |
| R505 | Gather sources recursively from the unmet difference, bounded by depth and terminating at a fixpoint. | Implemented by `src/summarization/gathering.rs` (`GatheringPlan`, `gather`); covered by three gathering tests including a citation cycle and an endless chain. |
| R506 | Cache every fetch content-addressed and per URL, so a warm cache replays byte-identically without reaching the provider or inheriting another URL's tier. | Implemented by `SourceCache` (digest-keyed bodies, one entry per URL); covered by `a_warm_cache_replays_the_same_gathering_without_fetching`. |
| R507 | Recheck before presenting: an unsupported statement is withheld from the summary while staying in the context. | Implemented by `src/summarization/recheck.rs` and `MergedContext::checked_summary`; covered by `a_statement_no_trusted_source_asserts_is_withheld_but_kept`. |
| R508 | Merge into a `world_model::Context`: a probability per statement, contradictions as mutual `Contradicts` edges, disagreement reported rather than resolved, and no fabricated consensus between two original sources that disagree. | Implemented by `src/summarization/context.rs` plus the cycle-collapse fixpoint in `Context::recalculate`; covered by `contradictions_become_contradicts_edges_and_are_reported_as_disagreement` and `a_saturated_mutual_contradiction_settles_at_maximal_uncertainty`. |
| R509 | Extend the summarization ladder downward with an identifier rung honouring syntactic constraints, a length budget, and naming conventions. | Implemented by `src/summarization/identifier.rs` and `SummarizationMode::Identifier`; covered by `the_identifier_rung_produces_valid_identifiers_under_a_length_budget` and `the_identifier_rung_is_the_bottom_of_the_ladder`. |
| R510 | Run the Stack Overflow case end to end, deterministically and without neural inference (`NON-GOALS.md:7`, `GOALS.md:54`), with case-study traceability and release metadata. | Implemented by `examples/issue_844_statement_merge.rs`, `docs/case-studies/issue-844/`, and the changelog fragment; covered by `the_stack_overflow_case_works_end_to_end`, `the_merge_is_deterministic_and_independent_of_source_order`, and `tests/unit/docs_requirements_issue_844.rs`. |
