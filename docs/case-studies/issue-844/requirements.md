# Issue #844 Requirements

The issue's six deliverables and eight acceptance criteria are kept one-to-one
with regression tests in `tests/unit/issue_844_statement_merge.rs`. R844-10
collects the whole task — worked example, traceability, release metadata — into
one requirement, because the issue asks for a pipeline, not six isolated
functions.

| ID | Requirement | Evidence | Regression test |
| --- | --- | --- | --- |
| R844-01 | Deduplicate at the statement level over the links network, conservatively, so N sources asserting one fact become one fact and every merge is an explainable, reversible link. | `src/summarization/dedup.rs` | `n_sources_asserting_one_fact_yield_one_statement_with_a_justification_link` |
| R844-02 | Merge wording differences only: extra content is a different fact, and no stemming is applied, because over-merging is forbidden by `NON-GOALS.md:39`. | `src/summarization/dedup.rs` | `wording_differences_merge_but_extra_content_does_not`, `inflected_wordings_stay_separate_because_the_merge_does_not_stem` |
| R844-03 | Make every merge reversible: retracting the recorded justification splits a conflating merge back into its variants. | `src/summarization/dedup.rs` (`DedupReport::split`) | `a_merge_that_conflates_two_facts_can_be_split` |
| R844-04 | Weight importance by evidence — the kind prior combined with observed link frequency and source stance — so an unoriginal mirror adds no probability. | `src/summarization/importance.rs` | `ranking_reflects_observed_frequency_and_source_stance`, `an_unoriginal_mirror_adds_no_probability` |
| R844-05 | Gather sources recursively from the unmet difference, terminating by fixpoint, respecting the depth bound, and stopping once nothing is missing. | `src/summarization/gathering.rs` | `recursive_gathering_terminates_by_fixpoint_over_a_citation_cycle`, `gathering_respects_the_depth_bound_on_an_endless_chain`, `gathering_stops_once_the_unmet_difference_is_empty` |
| R844-06 | Cache every fetch content-addressed and per URL, so a warm cache replays a byte-identical gathering without reaching the provider and without inheriting another URL's tier. | `src/summarization/gathering.rs` (`SourceCache`) | `a_warm_cache_replays_the_same_gathering_without_fetching` |
| R844-07 | Recheck before presenting: a statement no trusted source asserts is withheld from the summary but kept in the context. | `src/summarization/recheck.rs` | `a_statement_no_trusted_source_asserts_is_withheld_but_kept` |
| R844-08 | Merge into a context, not a list: every statement carries a probability, contradictions become `Contradicts` edges, and two original sources that flatly disagree settle at maximal uncertainty instead of a fabricated consensus. | `src/summarization/context.rs`; `src/world_model.rs` | `contradictions_become_contradicts_edges_and_are_reported_as_disagreement`, `a_saturated_mutual_contradiction_settles_at_maximal_uncertainty` |
| R844-09 | Extend the ladder downward with an identifier rung that honours syntactic constraints, a length budget, and a naming convention, and stays reachable from a merged context. | `src/summarization/identifier.rs`; `src/summarization/mod.rs` | `the_identifier_rung_produces_valid_identifiers_under_a_length_budget`, `the_identifier_rung_is_the_bottom_of_the_ladder` |
| R844-10 | Complete workflow: the Stack Overflow case works end to end, the result is deterministic and order-independent with no neural inference (`NON-GOALS.md:7`, `GOALS.md:54`), and the task is traceable through a case study, requirement ids, a worked example, and release metadata. | `examples/issue_844_statement_merge.rs`; `test-logs/example-output.txt`; `README.md`; `REQUIREMENTS.md`; changelog fragment | `the_stack_overflow_case_works_end_to_end`, `the_merge_is_deterministic_and_independent_of_source_order`, `tests/unit/docs_requirements_issue_844.rs` |

## Acceptance

All seventeen tests in `tests/unit/issue_844_statement_merge.rs` and all four in
`tests/unit/docs_requirements_issue_844.rs` must pass in one run. The
documentation tests parse the table above: a requirement whose named regression
test does not exist fails the build, so this map cannot drift away from the code.

## Out of scope, and why

Issue #844 declares itself blocked by #702 (retrieval planning) and #843 (real
HTTP fetching), both still open. Fetching therefore goes through the
`SourceProvider` trait: the gathering loop, its depth bound, its fixpoint, and
its content-addressed cache are shipped and tested, while the transport behind
them stays #843's job. Nothing in this change performs network I/O.
