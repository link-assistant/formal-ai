# Case study - Issue #441: "Unknown prompt: Что такое vulkan layer"

- **Issue:** [#441](https://github.com/link-assistant/formal-ai/issues/441)
- **Reported version:** 0.190.0 (WASM worker), GitHub Pages, mobile Firefox
- **Reported prompt:** `Что такое vulkan layer`
- **Reported result:** `intent: unknown`
- **Pull request:** [#473](https://github.com/link-assistant/formal-ai/pull/473)
- **Raw data:** [`raw-data/`](./raw-data/) contains the issue JSON, issue comments, and PR metadata captured for this fix.

## What Happened

The prompt starts with the Russian definition cue `Что такое`, then asks about
the Latin technical term `vulkan layer`. The reported reasoning trace showed the
formalization was correct:

```text
(@USER OP:define ?vulkan layer)
```

But the worker detected `language:en` because the Latin letters in `vulkan layer`
outnumbered the Cyrillic letters in `Что такое`. The prompt then fell through the
lookup chain and returned the English unknown-intent fallback.

## Root Cause

`src/language.rs` and the mirrored browser worker detector both used raw script
letter counts. That works for monolingual prompts, but fails for localized
definition prompts that include Latin product names, APIs, packages, or graphics
terms. In these cases the leading prompt language carries the user's command,
while the Latin span is the thing being asked about.

The Rust and JS paths had the same behavior, so the fix had to preserve parity.

## Fix

Language detection now records the first alphabetic script and checks for
supported-language question markers. When a prompt starts in a supported
non-Latin script, or when a term-first prompt includes local question words, the
detector preserves that prompt language even if the Latin term has more letters.
The existing dominant-script fallback still handles monolingual and other
mixed-script cases.

The change is mirrored in:

- `src/language.rs`
- `src/web/formal_ai_worker.js`

## Verification

Added regression coverage:

- `detects_language_from_mixed_script_definition_prompt` pins
  `Что такое vulkan layer` and term-first mixed-script variants as the expected
  supported languages in the Rust language contract.
- `tests/e2e/tests/issue-441.spec.js` drives the exact prompt through the web UI
  with a mocked Wikipedia REST summary and asserts the first summary lookup goes
  to `ru.wikipedia.org`, the answer is `intent:wikipedia_lookup`, and evidence
  contains `language:ru`.
- The existing multilingual fuzzy-search e2e case now asserts the localized
  closest-match note for mixed-script Russian, Hindi, and Chinese prompts.

The first attempted targeted Rust run in this workspace did not reach the test
assertion because dependency compilation filled the container filesystem with
`No space left on device`; generated build output was removed before continuing.
