## Issue #893 Iterative Summarization Validation and the 80% Quality Ratchet

Issue [#893](https://github.com/link-assistant/formal-ai/issues/893) (child of
[#710](https://github.com/link-assistant/formal-ai/issues/710)) records the
audit verdict *still-broken* for the part of issue
[#563](https://github.com/link-assistant/formal-ai/issues/563) that the file and
folder summarizers never covered. #563 did not only ask for
`summarize_repository_file`; it asked for a *protocol*: take two random
repository files, check the summaries, generalize, take two more, and repeat
until the result is stable on files nobody optimized for, at a quality bar of at
least 80%. The pipeline had the recursion, the exact captures and the
determinism, but nothing sampled random files, nothing iterated, and no metric
existed to be 80% of. The protocol lives in `src/summarization/validation/`
(`mod.rs` the loop and the reports, `sampling.rs` the draw, `criteria.rs` the
checks, `baseline.rs` the ratchet),
the operator surface in `src/cli_summarization.rs`, the committed baseline in
`data/summarization/quality-baseline.lino`, and the analysis in
`docs/case-studies/issue-893/`. One test per requirement, plus a whole-task test
that runs the protocol over the real repository, lives in
`tests/unit/specification/issue_893_summarization_validation.rs`.

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R893-1 | Define a reproducible seeded sampling protocol over repository files. | Implemented: `SamplingProtocol { seed, files_per_iteration, max_iterations, minimum_iterations, stability_window, stability_tolerance_percent }` sorts the corpus and permutes it with a seeded `splitmix64` Fisher-Yates shuffle, so the draw depends on the seed and the corpus alone — not on caller order — and is a permutation, so no file repeats inside a run. `stratified_sampling_order` then promotes the first fence-carrying Markdown file to the front, leaving every other file at its seeded position, because a uniform draw of the affordable size can miss that stratum entirely. `issue_893_seeded_sampling_is_reproducible_and_seed_dependent` asserts reproducibility, caller-order independence, seed dependence and the permutation property. |
| R893-2 | Validate two files per iteration until the result stabilizes or a reported bound is reached. | Implemented: `DEFAULT_FILES_PER_ITERATION = 2`; `validate_repository_summarization` draws disjoint two-file slices of the permutation and stops when `DEFAULT_STABILITY_WINDOW = 3` consecutive iterations all clear the ratchet within `DEFAULT_STABILITY_TOLERANCE_PERCENT = 5` points of one another — but never before `DEFAULT_MINIMUM_ITERATIONS = 12` iterations (24 files) have run, capped by what the corpus can supply, since three perfect iterations are six files and six files are no evidence about a corpus of thousands. Otherwise it stops at `max_iterations` with `bound_reached true`. `issue_893_iterations_validate_two_files_each_until_stable_or_bounded` asserts both exits and the minimum sample. |
| R893-3 | Define and publish the quality metric with an 80 percent minimum ratchet. | Implemented: `CRITERIA` publishes ten named, described criteria; `QualityScore` is an exact integer `passed/applicable` ratio, floored, with an empty score scoring 0 rather than a vacuous 100; `QUALITY_RATCHET_PERCENT = 80` and `ratchet_violations` enforce the floor plus monotonicity against `data/summarization/quality-baseline.lino`. `formal-ai summarization criteria` prints the published metric. `issue_893_quality_metric_is_published_and_ratcheted_at_eighty_percent` and `issue_893_committed_baseline_records_the_measured_run` assert it. |
| R893-4 | Exercise recursive Markdown embedded grammars through the production summarizer. | Implemented: `evaluate_file` scores every sampled file through `formalize_repository_file` / `RepositoryFileFormalization::summary`, and the `embedded_grammar_recursion` criterion checks every fenced block against an *independent* CommonMark fence scanner so the summarizer never grades itself. A run may not declare stability until at least one embedded grammar block has been exercised, and `ratchet_violations` rejects a run that recorded none. Because that rejection is fatal, reaching the recursive case is not left to luck: the stratified draw puts a fence-carrying Markdown file into iteration 0. Optional concrete-syntax evidence is bounded to 32 KiB per file or embedded block, while structural summarization still processes the complete artifact, so a seeded draw of a multi-megabyte trace cannot monopolize validation. `issue_893_markdown_embedded_grammars_run_through_the_production_summarizer` and `issue_893_oversized_structured_files_skip_the_unbounded_meta_language_parse` cover both paths. |
| R893-5 | Report honestly rather than claiming a stability the run never observed. | Implemented: `ValidationReport` carries `stabilized` and `bound_reached` separately, records every failing criterion with its evidence detail, and `to_links_notation` writes exactly what the ratchet reads back, including `ratchet_runner`, `ratchet_policy` and `honesty_policy`. Criteria that do not apply to a file are excluded from its denominator instead of counted as passes. |
