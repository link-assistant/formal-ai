# Issue 394 Case Study

## Summary

Issue: https://github.com/link-assistant/formal-ai/issues/394
Pull request: https://github.com/link-assistant/formal-ai/pull/397

The reported chat used Russian in the last user message, but the unknown-answer recovery guide mixed Russian prose with English rule-configuration commands (`List behavior rules`, `Show behavior rule unknown`, `When I say ... answer ...`) and described the missing behavior as "Links Notation rules." The fix keeps Russian unknown and capability guidance in Russian and uses "links rules" for user-facing behavior rules while reserving "Links Notation" for the storage/encoding format.

## Timeline

- 2026-06-04T17:42:22Z: issue #394 opened.
- 2026-06-04T19:05:39Z: issue #394 updated with screenshot/details.
- 2026-06-04T19:58:20Z: draft PR #397 opened from branch `issue-394-ffc1c872e3ab`.
- 2026-06-04: raw issue/PR data, screenshot, CI snapshot, online research, and local validation logs archived under this directory.

## Requirements Interpreted

- Unknown-route answers should default to the same language as the last user message when there is no matching links rule.
- Russian recovery guidance should explain how to list rules, inspect the `unknown` rule, and teach a dialog-local rule using Russian examples.
- User-facing text should say "links rules" or "правила связей" rather than "Links Notation rules"; "Links Notation" remains appropriate when describing the serialized format.
- The issue artifacts, screenshot, logs, and research trail should be saved under `docs/case-studies/issue-394/`.

## Root Cause

- The seed unknown response for Russian was localized only partially: the main prose was Russian, but the rule-management examples stayed in English.
- The unknown-reasoning fallback had its own hardcoded hint, so high-rigor unknown prompts bypassed the seed text and still emitted English rule examples.
- A nonlocalized trace sentence was appended to non-English unresolved unknown answers when the solver did not ask a clarification question.
- Existing tests verified that Russian unknown answers contained Cyrillic, but did not assert that the recovery commands stayed in Russian or that "Links Notation rules" was avoided.

## Fix

- Updated unknown-response seed copy and hardcoded fallback copy to describe local links rules.
- Localized Russian unknown-reasoning rule examples to `Покажи правила поведения`, `Покажи правило unknown`, and `Когда я скажу ... ответь ...`.
- Localized the unresolved-unknown trace note and missing-rule report path for Russian answers.
- Updated Russian capability/help text in Rust and the web worker.
- Updated self-facts text to describe "local links rules and seed facts."
- Added regression tests for the Russian unknown answer, Russian unknown-reasoning hint, and Russian capability command examples.
- Updated existing tests and e2e markers to match the new "local links rules" wording.

## Online Research

Research notes are archived in `raw-data/online-research.md`.

- Unicode UAX #24 documents script properties for text processing and notes that Russian is written with Cyrillic script: https://www.unicode.org/reports/tr24/
- MDN documents Unicode property escapes, including `Script` and `Script_Extensions`, and shows matching Cyrillic with `\\p{sc=Cyrillic}`: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Regular_expressions/Unicode_character_class_escape

The project already detects Russian for the reported prompt; the research supports treating a Cyrillic/Russian prompt as a strong signal for same-language recovery guidance.

## Artifacts

- Raw issue: `raw-data/issue-394.json`
- Raw issue comments: `raw-data/issue-394-comments.json`
- Raw PR: `raw-data/pr-397.json`
- PR comments/reviews: `raw-data/pr-397-*.json`
- Initial CI snapshot: `raw-data/ci-runs-initial.json`
- Original screenshot: `screenshots/issue-394-original.jpg`
- Validation logs: `raw-data/cargo-*.log`, `raw-data/e2e-check-*.log`, `raw-data/npm-ci-e2e.log`

## Validation

- `scripts/sync-seed.sh --check`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo test russian_unknown`: passed.
- `cargo test russian_capabilities`: passed.
- `cargo test --test unit chat_surface --verbose`: passed; log saved.
- `cargo test --all-features --verbose`: passed; log saved.
- `cargo test --doc --verbose`: passed; log saved.
- `cargo clippy --all-targets --all-features`: passed; log saved.
- `npm ci --prefix tests/e2e`: passed; log saved.
- `npm run --prefix tests/e2e check:i18n`: passed; log saved.
- `npm run --prefix tests/e2e check:language-parity`: passed; log saved.
- `npm run --prefix tests/e2e check:language-test-coverage`: passed; log saved.
- `npm run --prefix tests/e2e check:intent-coverage`: passed; log saved.
- `npm run --prefix tests/e2e check:web-tdz`: passed; log saved.
- `rust-script scripts/check-file-size.rs`: not run locally because `rust-script` is not installed; log saved with `command not found`.
