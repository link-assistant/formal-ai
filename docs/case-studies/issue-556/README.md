# Issue 556 Case Study: Response-Language Follow-up for Repository Lookups

## Summary

Issue [#556](https://github.com/link-assistant/formal-ai/issues/556) reported a browser conversation
where a Russian user asked for a code review of
`https://github.com/netkeep80/anum_docs`, received a repository-lookup answer in English, then wrote
`я не понимаю по английски, напиши по русски`. The solver treated that follow-up as unknown instead
of re-answering the previous repository lookup in Russian.

The fix recognizes response-language follow-ups through the seeded
`response_language_marker` meanings, recovers the prior user request from conversation history, and
rerenders the same repository lookup in the requested seeded language. Rust and the JavaScript wasm
worker mirror now both handle the path before clarification or unknown fallback.

## Collected Data

Fresh GitHub evidence is preserved in `raw-data/`:

- `issue-556.json` and `issue-556-comments.json` - source issue and maintainer guidance.
- `issue-526.json` - related translation-quality issue referenced by the maintainer comment.
- `pr-599.json`, `pr-599-conversation-comments.json`, `pr-599-review-comments.json`, and
  `pr-599-reviews.json` - prepared PR state and all PR comment surfaces.

## Requirements

| ID | Requirement | Source | Solution in this PR |
| --- | --- | --- | --- |
| R556-1 | Reproduce the reported Russian follow-up after a GitHub repository lookup. | Issue body | Add Rust regression coverage for the exact prompt sequence and a Playwright regression for the browser worker. |
| R556-2 | Generalize beyond the one Russian string. | Maintainer comment | Use seeded `response_language_marker` meanings rather than a hard-coded phrase check. |
| R556-3 | Cover all seeded supported languages for this class. | Maintainer comment plus issue #526 | Add Rust cases for English, Russian, Hindi, and Chinese response-language follow-ups. |
| R556-4 | Preserve the meaning of the prior request through translation. | Issue #526 reference | Reuse the prior user turn as the lookup source and only change the renderer target language. |
| R556-5 | Keep Rust and browser worker behavior aligned. | `CONTRIBUTING.md` | Mirror detection and dispatch in `src/web/worker/formal_ai_worker_*.js`. |

## Root Cause

The project lookup handler only inspected the current prompt. A follow-up such as
`напиши по русски` contains a language preference but no repository URL, so the direct project lookup
path could not reconstruct `netkeep80/anum_docs`. The clarification and unknown handlers then saw a
short meta-language request and produced the wrong intent.

The missing abstraction was a contextual response-language follow-up: "answer the previous user
request again, but in this target language." Translation marker data already existed in
`data/seed/meanings-translation.lino`, including response-language markers for English, Russian,
Hindi, and Chinese. The solver just did not apply those meanings to conversation history.

## Solution

The Rust solver now exposes `detect_response_language`, backed by the seeded
`ROLE_RESPONSE_LANGUAGE_MARKER` data. A new contextual handler runs early in dispatch:

1. detect a requested response language in the current prompt;
2. recover the previous user turn from conversation history;
3. try the project lookup handler against that previous turn;
4. render the answer in the requested target language;
5. keep evidence such as `response_language_followup:target` and `language_to` in the trace.

The browser worker mirrors the same behavior with `detectResponseLanguage`,
`tryProjectLookupForPrompt`, and `tryResponseLanguageFollowup`, so the wasm demo path no longer
diverges from Rust.

The implementation deliberately reuses the repository lookup renderer rather than translating a
finished answer string. That keeps the repository identity, platform label, and Links Notation
evidence grounded in the original request while allowing the response language to vary.

## Verification

Focused regressions added by this PR:

- `tests/unit/specification/project_lookups.rs` covers the exact Russian report and seeded
  English, Russian, Hindi, and Chinese response-language follow-ups.
- `tests/e2e/tests/issue-556.spec.js` verifies the browser worker reanswers the prior GitHub lookup
  in Russian and does not fall back to the English repository sentence.

Local verification commands for PR finalization:

```bash
cargo test --test unit issue_556 -- --nocapture
cd tests/e2e && npx playwright test tests/issue-556.spec.js --config=playwright.local.config.js
cargo fmt --all -- --check
cargo clippy --all-targets --all-features
rust-script scripts/check-file-size.rs
```
