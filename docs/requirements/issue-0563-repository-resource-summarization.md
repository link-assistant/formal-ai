## Issue #563 Repository Resource Summarization

Issue [#563](https://github.com/link-assistant/formal-ai/issues/563) asks the
summarization pipeline to handle arbitrary repository files, formalize each file
before summarizing it, recurse into Markdown embedded grammars, and preserve the
issue research under `docs/case-studies/issue-563`. PR
[#564](https://github.com/link-assistant/formal-ai/pull/564) adds a repository
file formalization boundary on top of the existing deterministic
`formalize -> summarize -> deformalize` pipeline. New rows R345-R354 capture
the issue-specific obligations.

Review feedback on PR #564 (konard) made the generalization explicit: the
solution must not stay file-specialized, but follow the meta algorithm —
describe the task, formalize it to the meta language, decompose, solve, and
compose — so that *any* repository resource can be summarized, **including
folders**, not only files. Rows R355-R359 capture this generalization: a
directory is summarized by the recursive decompose -> summarize -> compose loop
(`src/summarization/resource.rs`), with recursion depth bounded by the
summarization mode ladder.

| ID | Requirement | Status |
| --- | --- | --- |
| R345 | The system must summarize arbitrary files from this repository, not only existing curated summarization inputs. | Implemented by `src/summarization/file.rs::{formalize_repository_file,summarize_repository_file}` and public re-exports from `src/lib.rs`; covered by `tests/unit/specification/summarization_pipeline.rs::repository_file_summary_recurses_into_markdown_embedded_grammars`. |
| R346 | The implementation should start from two random repository files, summarize them manually, and generalize the algorithm beyond those exact files. | Implemented by `docs/case-studies/issue-563/raw-data/random-files-sampled.txt` and `manual-random-file-summaries.md`; the code handles generic code, structured data, Markdown, and fallback text rather than hardcoding the sampled JSON files. |
| R347 | Each file must be formalized before summarization, using the project's meta-language orientation. | Implemented by `RepositoryFileFormalization` and `RepositoryFileFormalization::links_notation()`, with `MetaLanguageFormalization` parser evidence for supported grammars. |
| R348 | Markdown files must be handled recursively with multiple embedded grammars. | Implemented by `EmbeddedGrammarFormalization` records for fenced code blocks, including CommonMark EOF-close behavior; covered by `formalize_repository_file_markdown_records_embedded_grammars` and `formalize_repository_file_markdown_closes_embedded_grammar_at_eof`. |
| R349 | Summarization must remain part of the meta algorithm, not a detached ad hoc formatter. | Implemented by routing repository-file content statements through the existing `SummarizationConfig`, `summarize`, and `deformalize` stages. |
| R350 | Recursive reasoning steps and evidence must be inspectable. | Implemented by the link-native `repository_file` rendering that exposes metadata, statements, embedded grammars, and parser evidence. |
| R351 | Every described behavior needs tests. | Implemented by the unit and source tests above plus `tests/source/source_tests/summarization/mod/tests.rs::formalize_repository_file_rust_records_meta_language_and_symbols` and `tests/unit/docs_requirements_issue_563.rs::issue_563_repository_file_summarization_documents_are_traceable`. |
| R352 | Issue data and analysis must be preserved under `docs/case-studies/issue-563`. | Implemented by `docs/case-studies/issue-563/README.md` and raw-data captures for the issue, PR, comments, CI snapshot, code search, random-file sample, manual summaries, and online research. |
| R353 | Online research and existing components/libraries must be checked. | Implemented by `docs/case-studies/issue-563/raw-data/online-research.md`, which surveys CommonMark, Tree-sitter, GitHub Linguist, and local summarization/meta-language components. |
| R354 | Everything must land in the single prepared PR #564. | Implemented by this branch and documented in `docs/case-studies/issue-563/README.md`. |
| R355 | Summarization must generalize beyond files to any repository resource, including folders, instead of staying a file-specialized solution. | Implemented by `src/summarization/resource.rs::{RepositoryEntry,formalize_repository_resource,summarize_repository_resource}`, re-exported from `src/lib.rs`; covered by `tests/unit/specification/summarization_pipeline.rs::summarize_repository_resource_subsumes_file_summarization`. |
| R356 | A folder must be summarized by the meta algorithm: decompose into children, summarize each child, then compose the child summaries with aggregate metadata. | Implemented by `RepositoryDirectoryFormalization::summary` (decompose -> summarize -> compose) and `formalize_repository_directory` (recursive file/subdirectory/line/byte aggregates); covered by `directory_summary_reports_recursive_aggregate_counts` and `formalize_repository_resource_distinguishes_files_and_directories`. |
| R357 | Recursion over nested folders must be bounded so deep trees stay summarizable, using the summarization mode ladder rather than an ad hoc depth cap. | Implemented by `SummarizationMode::one_step_shorter` and `child_summary_cap`, so a `Full` folder describes children in `Standard`, theirs in `Short`, deeper as `Topic`; covered by `directory_summary_recurses_with_one_step_shorter_mode` and `directory_short_summary_bounds_listed_children`. |
| R358 | The generalized resource layer must stay deterministic and inspectable, with link-native evidence for folders as for files. | Implemented by `RepositoryDirectoryFormalization::links_notation` rendering a `repository_directory` block of paths, counts, and per-child kind; covered by `directory_links_notation_lists_children_by_kind`. |
| R359 | The folder generalization must be demonstrated and tested end to end. | Implemented by `examples/issue_563_folder_summary.rs` and the source-mirror tests `tests/source/source_tests/summarization/mod/tests.rs::{summarize_repository_resource_topic_directory_is_identity_only,summarize_repository_resource_full_directory_recurses_into_nested_folder}`. |
