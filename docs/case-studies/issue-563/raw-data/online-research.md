# Online research - repository-file summarization

> Summarized and cited per the repository documentation non-goal: research notes
> should cite sources and avoid copying large external text.

## CommonMark fenced code blocks

CommonMark 0.31.2 defines a fenced code block as a run of at least three
backticks or tildes, records an optional info string on the opening line, and
treats the first word of that info string as the conventional language label.
The code block's content is literal text rather than Markdown inline content,
and an unclosed fence is closed by end-of-document.

Source: <https://spec.commonmark.org/0.31.2/#fenced-code-blocks>

Design implication for issue #563: a Markdown file is not one flat prose stream.
It may contain embedded grammars, so the file formalizer extracts fenced blocks,
normalizes the info-string language label, and formalizes each block separately
from the Markdown prose summary. The EOF-close rule is covered by
`formalize_repository_file_markdown_closes_embedded_grammar_at_eof`.

## Tree-sitter multi-language parsing

Tree-sitter describes itself as a parser generator and incremental parsing
library that can build concrete syntax trees, update them as source changes, and
remain useful in the presence of syntax errors. Its advanced parsing docs
explicitly model multi-language documents by parsing selected byte ranges with
different languages; the docs show one document represented by overlapping ERB,
HTML, and Ruby trees.

Sources:

- <https://tree-sitter.github.io/tree-sitter/>
- <https://tree-sitter.github.io/tree-sitter/using-parsers/3-advanced-parsing.html#multi-language-documents>

Design implication for issue #563: the long-term general solution can replace
or supplement simple symbol extraction with syntax-tree extraction. The current
PR stays dependency-light and reuses the existing `meta-language` crate already
in the repository for supported grammars; the shape of
`RepositoryFileFormalization` leaves room for richer parser-backed evidence.

## GitHub Linguist file-language detection

GitHub Linguist's FAQ lists file-language detection strategies in order:
modelines, known filenames, shebangs, known extensions, heuristic rules, then a
naive Bayesian classifier. Its public language API also exposes lookup by
filename, extension, alias, interpreter, and id.

Sources:

- <https://github.com/github-linguist/linguist/issues/4263>
- <https://github.com/github-linguist/linguist/blob/main/lib/linguist/language.rb>

Design implication for issue #563: this PR implements the conservative first
slice of the Linguist approach: known filenames plus common extensions, with
Markdown info strings used for embedded grammars. Shebang/modeline/heuristic
classification is a natural future extension if repository-file summarization
needs broader coverage.

## Existing local components surveyed

Local search found these reusable components:

- `src/summarization/` already provides deterministic
  `formalize -> summarize -> deformalize`, Markdown README cleaning, dialog
  weighting, and topic-title generation.
- `meta_language::LinkNetwork` with `NetworkProjection` and
  `ParseConfiguration` already parses supported source/data languages and
  reconstructs source text.
- `links_format::sanitize_lino_value` already renders safe Links Notation field
  values.
- `tests/source/` mirrors source modules, so new summarization code must be
  mirrored and covered there.

Raw local evidence:

- `local-summarization-survey.txt`
- `code-search-summarization.txt`
- `code-search-meta-language.txt`
- `recent-merged-summarization-prs.json`
- `recent-merged-meta-language-prs.json`
