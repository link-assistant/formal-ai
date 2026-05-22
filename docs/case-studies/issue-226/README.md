# Issue 226 Case Study: Wikipedia Article-Existence Questions

## Prompt

Reported prompt:

```text
согласованность в предложении - есть такая статья в википедии?
```

Follow-up prompt that exposed the second failure:

```text
Google Gemini ответил мне так - В русскоязычной Википедии нет отдельной статьи с названием «Согласованность в предложении», однако ... Согласование (грамматика) ...
можешь научиться делать так же?
```

## Raw Data

Saved artifacts are in `raw-data/`:

- `issue-226.json`, `issue-226-comments.json`
- `pr-227.json`, `pr-227-conversation-comments.json`, `pr-227-review-comments.json`, `pr-227-reviews.json`
- `ci-runs-before.json`
- `ruwiki-search-*.json` and `ruwiki-summary-soglasovanie-grammar.json`
- `github-code-search-wikipedia-article.json`
- `e2e-before.log`, `e2e-after.log`, `e2e-full.log`
- `cargo-*.log`, `check-*.log`, `git-diff-check.log`

There were no issue comments or PR review/conversation comments when this case study was captured.

## External Research

Online search and Wikimedia REST API checks showed:

- The exact phrase `согласованность в предложении` does not resolve to a dedicated ru.wikipedia.org article.
- The Russian Wikipedia page `Согласование (грамматика)` exists and describes agreement as a type of subordinate syntactic relation.
- Wikimedia search for `согласованность грамматика` ranks `Согласование (грамматика)` first.
- Wikimedia search for the whole phrase plus `грамматика` ranks sentence-level pages higher, so the implementation strips the sentence-context phrase and searches the grammatical concept with grammar context.

Reference URLs:

- https://ru.wikipedia.org/wiki/Согласование_(грамматика)
- https://ru.wikipedia.org/w/rest.php/v1/search/page?q=согласованность%20грамматика&limit=5
- https://ru.wikipedia.org/w/rest.php/v1/search/page?q=согласование%20в%20предложении&limit=5

## Root Cause

Two independent bugs produced the reported dialogue:

1. `tryWikipediaLookup` only handled concept-question shapes such as `что такое X`, `who is X`, or `как устроен X`. The reported article-existence question did not enter the Wikipedia lookup pipeline, so it fell through to the unknown-answer fallback.
2. `recognizeInterfaceCommand` treated any message containing a language word plus a target language word as a UI preference command. Quoted prose containing `русскоязычной`, `русского языка`, and `Википедии` was misclassified as `configure_language`, producing `Done. UI language is now ru.`

## Fix

The web worker now has a dedicated `wikipedia_article_question` handler before generic Wikipedia lookup. It:

- Detects English/Russian article-existence questions that mention Wikipedia and an article/page.
- Extracts the candidate title from dash-separated prompts and direct question forms.
- Checks for an exact Wikipedia article first.
- For Russian grammar phrases like `X в предложении`, searches the core concept `X` with `грамматика` context.
- Renders a sourced answer that distinguishes exact articles from closest useful matches.

The UI command recognizer now requires an explicit command shape for UI language changes, such as `Switch UI language to Russian` or `переключи язык на русский`. Quoted text that merely discusses Russian language content no longer mutates preferences.

## Reproduction And Verification

Before the fix, focused Playwright tests failed as expected:

```text
npm --prefix tests/e2e run test:local -- -g "Issue #226"
```

See `raw-data/e2e-before.log`:

- Quoted Gemini prose returned `Done. UI language is now ru.`
- The article-existence prompt returned the unknown fallback.

After the fix, the same focused suite passed:

```text
2 passed
```

See `raw-data/e2e-after.log`.

The full local browser suite also passed after the implementation:

```text
170 passed
```

See `raw-data/e2e-full.log`.

## Regression Tests

- `tests/e2e/tests/multilingual.spec.js`: verifies the Russian article-existence question returns a sourced closest match for `Согласование (грамматика)` instead of the unknown fallback.
- `tests/e2e/tests/demo.spec.js`: verifies quoted Russian prose does not change UI language, while an explicit language command still works.
