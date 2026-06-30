# Issue 535: Text attachment originality checks

> **Status:** Implemented in PR #598.
> **Type:** Bug fix for multilingual attachment handling and grounded web-search routing.

- **Issue:** <https://github.com/link-assistant/formal-ai/issues/535>
- **Pull request:** <https://github.com/link-assistant/formal-ai/pull/598>
- **Raw data:** [`raw-data/`](raw-data/)

## Summary

Issue #535 reported that the Web app answered unknown for the Russian prompt
`Проверь данный текст на уникальность и на плагиат` when a text/plain file
named `variation-tech-model-manual.txt` was attached.

The maintainer clarified that this should generalize beyond the exact prompt:
Desktop, Telegram, Web app, and other surfaces need attached-file plagiarism or
originality checks across languages, routed through the general meta-algorithm
and grounded external data sources.

## Root Cause

The browser UI sent only attachment metadata to the solver:

```text
Attached files:
1. variation-tech-model-manual.txt (text/plain, 160.3 KB)
```

The text/plain body was not sampled into solver context, so the worker and Rust
solver could only see a file name. The solver also had no dedicated intent for
document originality or plagiarism checks, so the Russian request missed the web
research path and fell through to `unknown`.

## Fix

The implementation adds a mirrored Rust and browser-worker handler for
`document_originality_check`. The handler recognizes action, subject, and
document roles from seed meanings instead of a single hardcoded phrase. It logs
attachment evidence, requests local-file reading evidence for uploaded files,
and routes the request to the existing grounded web-search provider list with a
new query kind.

The Web app now samples readable text attachments before sending a prompt. For
plain text, Markdown, JSON, code, logs, and similar text formats, the solver
context includes a bounded `Text excerpt:` block and records whether the sample
was truncated or unavailable.

The seed corpus now declares multilingual meanings for originality/plagiarism
actions and document subjects in English, Russian, Hindi, and Chinese, and the
role registry was regenerated from those meanings.

## Evidence

Raw GitHub snapshots are preserved in [`raw-data/`](raw-data/):

| File | Contents |
|---|---|
| [`issue-535.json`](raw-data/issue-535.json) | Issue title, body, metadata, and embedded comments from `gh issue view`. |
| [`issue-535-comments.json`](raw-data/issue-535-comments.json) | Paginated issue comments. |
| [`pr-598.json`](raw-data/pr-598.json) | Prepared PR metadata before final update. |
| [`pr-598-conversation-comments.json`](raw-data/pr-598-conversation-comments.json) | PR conversation comments. |
| [`pr-598-review-comments.json`](raw-data/pr-598-review-comments.json) | Inline PR review comments. |
| [`pr-598-reviews.json`](raw-data/pr-598-reviews.json) | PR reviews. |

## Verification

The reproducing Rust tests were added before the fix and initially failed with
the reported unknown path. After implementation, the focused checks passed:

```text
cargo test issue_535 --test unit
2 passed; 0 failed

cargo test document_originality --test unit
2 passed; 0 failed

npx playwright test tests/issue-535.spec.js --config=playwright.local.config.js
1 passed
```

The Playwright regression uploads a text/plain attachment, sends the Russian
prompt, and asserts that the answer is not unknown, keeps Russian output, emits
`intent:document_originality_check`, records the attachment and local-file read
evidence, records `document_originality_check:text_sample:present`, and marks
the web-search query kind as `document_originality_check`.

Additional seed integrity checks passed:

```text
cargo test meaning_definition_references_resolve_to_defined_meanings --test unit
cargo test every_role_value_is_declared_in_the_registry --test unit
```

## Remaining Scope

This PR implements the deterministic routing, attachment-text visibility, and
grounded search workflow for the reported class of requests. Full claim-level
plagiarism scoring against external corpora and future relative-meta-logic
probability updates remain larger follow-up work beyond this unknown-response
bug.
