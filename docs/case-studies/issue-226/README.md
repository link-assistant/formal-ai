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
- `e2e-multilingual-before.log`, `e2e-multilingual-after.log`
- `cargo-*.log`, `check-*.log`, `git-diff-check.log`

There were no issue comments or PR review comments when this case study was captured. A later PR conversation comment asked for CI/CD rules and tests that guarantee English, Russian, Chinese, and Hindi support for each feature or fix.

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

The web worker now has a dedicated `wikipedia_article_question` handler before generic capability and Wikipedia lookup. It:

- Detects English, Russian, Hindi, and Chinese article-existence questions that mention Wikipedia and an article/page.
- Extracts the candidate title from dash-separated prompts and direct question forms.
- Checks for an exact Wikipedia article first.
- For sentence-context grammar phrases like `X в предложении`, `agreement in a sentence`, `वाक्य में X`, or `句子中的X`, searches the core concept with a language-matched grammar context.
- Renders a sourced answer that distinguishes exact articles from closest useful matches.

The UI command recognizer now requires an explicit command shape for UI language changes, such as `Switch UI language to Russian` or `переключи язык на русский`. Quoted text that merely discusses language content no longer mutates preferences.

The prompt-pattern seed and `check:intent-coverage` CI guard now require this issue's Wikipedia article-question and UI-language command cases to cover every supported language (`en`, `ru`, `hi`, `zh`).

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

After the PR feedback requested supported-language guarantees, the expanded focused suite first reproduced the missing Hindi article-question handling:

```text
Expected pattern: /लेख है/
Received string: "मैं समझ नहीं पाया। ..."
```

See `raw-data/e2e-multilingual-before.log`.

After the multilingual extension, the focused suite passed:

```text
2 passed
```

See `raw-data/e2e-multilingual-after.log`.

The full local browser suite also passed after the implementation:

```text
170 passed
```

See `raw-data/e2e-full.log`.

## Regression Tests

- `tests/e2e/tests/multilingual.spec.js`: verifies exact-title and sentence-context Wikipedia article-existence questions in English, Russian, Hindi, and Chinese.
- `tests/e2e/tests/demo.spec.js`: verifies quoted language prose does not change UI language, while explicit UI-language commands still work in English, Russian, Hindi, and Chinese.
- `tests/e2e/scripts/check-multilingual-intent-coverage.mjs`: fails CI if the issue-specific prompt-pattern and browser-test matrices do not cover every supported language.
