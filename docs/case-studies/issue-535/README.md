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

Following the maintainer's clarification in
[comment 4754747438](https://github.com/link-assistant/formal-ai/issues/535#issuecomment-4754747438),
the handler was generalized from plagiarism-only into a full **verification
class**. The action/subject/document cues were broadened to authenticity,
factual-accuracy, and veracity requests (`verify`, `authenticate`, `fact-check`,
`достоверность`, `सत्यता`, `真实性`, …) so the whole class of similar questions
routes to the same grounded workflow in every supported language.

Each statement in an attached document is now weighed with **relative-meta-logic**
(modelled on
[link-foundation/relative-meta-logic](https://github.com/link-foundation/relative-meta-logic)):
a statement starts from an assumed-true prior (0.6), its probability is *raised*
by trusted original-first sources (government/first-party at weight 1.0, original
journalism at 0.85, independent corroboration at 0.5) and *lowered* by
contradicting originals, while unoriginal reposts contribute no mass and are
recorded as ignored. The handler splits the sampled text into checkable
statements across scripts, builds a dedicated fact-check web-search query for
each, and replays the assessment into the append-only event log. This plan is
mirrored byte-for-byte into the Web app worker so the browser and the Rust engine
emit identical evidence.

Telegram document attachments are folded into the shared attachment-context
builder, so a forwarded file (with or without a caption) reaches the same
verification handler as the Desktop and Web surfaces.

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
8 passed; 0 failed

cargo test document_originality --test unit
3 passed; 0 failed

npx playwright test tests/issue-535.spec.js --config=playwright.local.config.js
1 passed
```

The unit suite now covers the reported prompt, one document-originality case per
supported language, the generalized verification class (authenticity/veracity in
every language), Telegram attachment routing, and per-statement
relative-meta-logic grounding
(`document_originality_grounds_each_statement_with_relative_meta_logic`).

The Playwright regression uploads a text/plain attachment, sends the Russian
prompt, and asserts that the answer is not unknown, keeps Russian output, emits
`intent:document_originality_check`, records the attachment and local-file read
evidence, records `document_originality_check:text_sample:present`, marks the
web-search query kind as `document_originality_check`, and confirms the browser
worker emits the relative-meta-logic evidence
(`relative_meta_logic:assumed_prior:0.600000`,
`relative_meta_logic:trusted_source_tier:original_first_party:weight=1.000000`,
`relative_meta_logic:ignored_source_tier:unoriginal`, and the per-statement
`statement_verification:*` links).

The `examples/issue_535_statement_verification.rs` example demonstrates the whole
plan end to end — statement extraction across scripts, grounding queries, and how
the assumed-true prior moves under each source tier.

Additional seed integrity checks passed:

```text
cargo test meaning_definition_references_resolve_to_defined_meanings --test unit
cargo test every_role_value_is_declared_in_the_registry --test unit
```

## Scope

This PR implements the deterministic routing, attachment-text visibility, the
generalized verification class across every supported language, per-statement
relative-meta-logic probability weighing, and the grounded search workflow — on
the CLI/HTTP, Telegram, and Web app surfaces, with the Rust engine and the
browser worker kept byte-for-byte identical.

The solver runs offline and deterministically, so the relative-meta-logic plan
records exactly *what* would be checked for each statement and *how* the
resulting evidence would move its probability, rather than performing live
network calls. Wiring real grounding results into `RelativeEvidence` (turning the
planned queries into fetched original-first sources at request time) is the
natural next increment; the probability machinery, source-trust taxonomy, and
per-statement query plan it consumes are all in place and tested here.
