# Issue 563 Case Study

> **Status:** Repository-file summarization implemented in PR #564.
> **Type:** Enhancement (new summarization API + documentation).

- **Issue:** <https://github.com/link-assistant/formal-ai/issues/563>
- **Pull request:** <https://github.com/link-assistant/formal-ai/pull/564>
- **Online research:** [`raw-data/online-research.md`](raw-data/online-research.md)
- **Manual random-file summaries:** [`raw-data/manual-random-file-summaries.md`](raw-data/manual-random-file-summaries.md)

All raw artifacts referenced below live in [`raw-data/`](raw-data/).

---

## 1. Summary

Issue #563 asks `formal-ai` to summarize arbitrary repository files, not only
curated project statements, README prose, dialogs, or chat titles. The key
technical requirement is that each file first becomes an auditable
meta-language-style representation, with Markdown handled recursively because a
Markdown document may contain multiple embedded grammars in fenced code blocks.

The implemented surface is:

- `formalize_repository_file(path, content)`, which returns a
  `RepositoryFileFormalization` record containing file metadata, statements,
  optional `meta-language` parser evidence, embedded grammar records, and a
  `links_notation()` rendering.
- `summarize_repository_file(path, content, config)`, which reuses the existing
  deterministic `formalize -> summarize -> deformalize` configuration and adds
  file-format, size, parser-evidence, and embedded-grammar context.

For Markdown, the implementation formalizes prose through the existing Markdown
summarizer and recursively extracts fenced code blocks. Each block receives its
own language label, statement count, and `meta-language` parse evidence when the
current grammar dependency supports that language. The whole file is still
rendered as link-native `repository_file`, `statement`, `embedded_grammar`, and
`meta_language` records, so callers can inspect the same data the prose summary
uses.

## 2. Collected Data

| File | What it is |
|---|---|
| [`raw-data/issue-563.json`](raw-data/issue-563.json) | Issue snapshot captured with `gh issue view 563`. |
| [`raw-data/issue-563-comments.json`](raw-data/issue-563-comments.json) | Issue comments; empty at implementation time. |
| [`raw-data/pr-564.json`](raw-data/pr-564.json) | Initial draft PR metadata before this implementation. |
| [`raw-data/pr-564-conversation-comments.json`](raw-data/pr-564-conversation-comments.json) | PR conversation comments; empty at implementation time. |
| [`raw-data/pr-564-review-comments.json`](raw-data/pr-564-review-comments.json) | Inline review comments; empty at implementation time. |
| [`raw-data/pr-564-reviews.json`](raw-data/pr-564-reviews.json) | PR reviews; empty at implementation time. |
| [`raw-data/recent-ci-runs.json`](raw-data/recent-ci-runs.json) | CI snapshot for the prepared branch before implementation. |
| [`raw-data/code-search-summarization.txt`](raw-data/code-search-summarization.txt) | GitHub code search for existing summarization components. |
| [`raw-data/code-search-meta-language.txt`](raw-data/code-search-meta-language.txt) | GitHub code search for meta-language usage. |
| [`raw-data/local-summarization-survey.txt`](raw-data/local-summarization-survey.txt) | Local `rg` survey of summarization and meta-language references. |
| [`raw-data/recent-merged-summarization-prs.json`](raw-data/recent-merged-summarization-prs.json) | Recent merged PRs touching summarization. |
| [`raw-data/recent-merged-meta-language-prs.json`](raw-data/recent-merged-meta-language-prs.json) | Recent merged PRs touching meta-language work. |
| [`raw-data/random-files-sampled.txt`](raw-data/random-files-sampled.txt) | Two random repository files sampled with `git ls-files | shuf -n 2`. |
| [`raw-data/manual-random-file-summaries.md`](raw-data/manual-random-file-summaries.md) | Manual summaries and expected generalized behavior for the sampled files. |
| [`raw-data/online-research.md`](raw-data/online-research.md) | Cited research on CommonMark fences, Tree-sitter multi-language parsing, and GitHub Linguist detection. |

## 3. Requirements

These requirements are extracted from the issue body. They are recorded in
[`REQUIREMENTS.md`](../../../REQUIREMENTS.md) as R345-R354.

| ID | Requirement | Status |
|---|---|---|
| R345 | The system must summarize arbitrary files from this repository, not only existing curated summarization inputs. | Implemented by `formalize_repository_file` and `summarize_repository_file` in `src/summarization/file.rs`, exported from `src/lib.rs`. |
| R346 | The implementation should start from two random repository files, summarize them manually, and generalize the algorithm beyond those exact files. | Implemented by `raw-data/random-files-sampled.txt` and `raw-data/manual-random-file-summaries.md`; the shipped algorithm handles generic code, structured data, Markdown, and fallback text rather than bespoke paths for the two sampled JSON files. |
| R347 | Each file must be formalized before summarization, using the project's meta-language orientation. | Implemented by `RepositoryFileFormalization::links_notation()` plus `MetaLanguageFormalization` parser evidence for supported grammars. |
| R348 | Markdown files must be handled recursively with multiple embedded grammars. | Implemented by fenced-code extraction into `EmbeddedGrammarFormalization` records; covered by tests for Rust and JavaScript fences plus an EOF-closed fence. |
| R349 | Summarization must remain part of the meta algorithm, not a detached ad hoc formatter. | Implemented by reusing `SummarizationConfig`, `summarize`, and `deformalize` for retained content statements. |
| R350 | Recursive reasoning steps and evidence must be inspectable. | Implemented by the link-native `repository_file` rendering that exposes file metadata, statements, embedded grammars, and parser evidence. |
| R351 | Every described behavior needs tests. | Implemented by specification tests and source tests for Markdown recursion, parser evidence, symbol extraction, and EOF fence handling. |
| R352 | Issue data and analysis must be preserved under `docs/case-studies/issue-563`. | Implemented by this case study and the `raw-data/` files listed above. |
| R353 | Online research and existing components/libraries must be checked. | Implemented by `raw-data/online-research.md`, code search captures, and the prior-art survey below. |
| R354 | Everything must land in the single prepared PR #564. | Implemented by this PR branch, with PR metadata updated after verification. |

## 4. Root Cause

The existing summarization pipeline was useful but narrow. It accepted free-form
prose, Markdown README text, dialogs, and curated project statements. It did not
have a repository-file boundary, so it could not preserve file path, size,
detected format, parser evidence, or embedded grammars before compression.

That omission made Markdown the clearest failure mode. The previous README path
intentionally stripped fenced code blocks as noise before summarizing prose. That
is right for README descriptions, but wrong for "summarize any file", where a
Markdown file's fenced Rust, JavaScript, shell, JSON, or other blocks are part of
the file's meaning and need their own recursive formalization records.

## 5. Implemented Design

`RepositoryFileFormalization` is the new boundary object. It contains:

- `path`, `format`, `line_count`, and `byte_count`;
- retained `Statement` values for the file's prose, symbols, or structural keys;
- `embedded_grammars` for Markdown fenced blocks;
- optional `meta_language` evidence with parser label, syntax-link count,
  total-link count, parse-error status, and text-preservation status.

Format detection follows the conservative part of GitHub Linguist's approach:
known filenames first, then common extensions. This PR intentionally does not
add shebang/modeline heuristics or a Bayesian classifier.

For code files, the formalizer extracts a bounded list of high-signal symbols
such as Rust functions/structs, JavaScript functions/classes/bindings, Python
functions/classes, and similar declarations for common languages. For structured
files, it records top-level keys. For everything else, it falls back to the
existing prose formalizer.

For Markdown files, the formalizer:

1. runs existing Markdown prose formalization for the main statements;
2. scans CommonMark-style fences using backtick/tilde marker length and EOF-close
   behavior;
3. normalizes the first info-string word into a language label;
4. recursively formalizes each fenced block as an embedded grammar;
5. attaches parser evidence through `meta-language` when the language is
   supported by the current dependency.

## 6. Prior Art And Existing Components

| Component | Relevance | Decision |
|---|---|---|
| Existing `src/summarization/` pipeline | Already provides deterministic statement ranking and output modes. | Reused directly; no parallel summarizer added. |
| `meta_language::LinkNetwork` | Parses supported source/data grammars and reconstructs text. | Reused for parser evidence on supported formats. |
| CommonMark fenced-code rules | Defines how Markdown embeds language-labeled literal blocks. | Mirrored in the lightweight fence scanner, including EOF-close behavior. |
| Tree-sitter | General parser framework with multi-language range parsing. | Kept as future option; not added because the current crate already has meta-language parser evidence and the issue can be solved without a new dependency. |
| GitHub Linguist | Production language-detection strategy for repository files. | Used as design reference; this PR implements filename/extension detection as the first conservative slice. |

## 7. Verification

The reproducing test was added before implementation:

- `tests/unit/specification/summarization_pipeline.rs::repository_file_summary_recurses_into_markdown_embedded_grammars`

The implementation is additionally covered by:

- `tests/source/source_tests/summarization/mod/tests.rs::formalize_repository_file_markdown_records_embedded_grammars`
- `tests/source/source_tests/summarization/mod/tests.rs::formalize_repository_file_markdown_closes_embedded_grammar_at_eof`
- `tests/source/source_tests/summarization/mod/tests.rs::formalize_repository_file_rust_records_meta_language_and_symbols`
- `tests/unit/docs_requirements_issue_563.rs::issue_563_repository_file_summarization_documents_are_traceable`

Local verification commands are recorded in PR #564 after implementation.

## 8. Risks And Follow-Ups

- Format detection is intentionally conservative. Shebangs, editor modelines,
  Linguist heuristic rules, and classifier-backed disambiguation can be added
  later without changing the public API.
- Symbol extraction is lightweight and deterministic. Tree-sitter-backed syntax
  extraction would improve precision for code-heavy summaries, but would add
  dependency and grammar management work.
- Markdown formalization records prose and fenced grammars; it does not claim a
  full CommonMark AST because the current `meta-language` dependency does not
  expose a Markdown parser label in this codebase.
- `summarize_repository_file` is a library surface. A future issue can decide
  how repository-file uploads, CLI commands, or web UI affordances should call it.
