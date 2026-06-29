# Case study - Issue #501: install how-to prompts

- **Issue:** [#501](https://github.com/link-assistant/formal-ai/issues/501)
- **Reported version:** 0.205.0 (WASM worker)
- **Reported prompt:** `how install cursor`
- **Reported result:** `intent: unknown`
- **Pull request:** [#587](https://github.com/link-assistant/formal-ai/pull/587)
- **Raw data:** [`raw-data/`](./raw-data/) contains the captured issue JSON and comments.

## Requirement

The prompt is a telegraphic procedural request: it omits the connector from
`how to install cursor`, but still has a known procedure action and an object.
It should not fall through to the unknown opener.

For install tasks, the source path should prefer product official documentation
or the official repository install page before community how-to sources, then
keep the existing recursive step/source checks and answer in the detected
language.

## Root Cause

Issue #481 added a narrow seed-backed path for elided prompts like
`how order ...`, but the approved action set did not contain `install`. The
extractor therefore rejected `how install cursor` even though the later
procedural handler was the right intent.

The browser worker also tried wikiHow before web search for all procedures. That
was acceptable for generic how-to prompts, but inverted the desired priority for
installation requests where official docs should be considered first.

## Fix

Added an `install` procedural action meaning in the shared how-to seed for
English, Russian, Hindi, and Chinese. Both the Rust handler and JS worker mirror
now recover install tasks from elided prompts and record:

- request: `install cursor`
- action: `install`
- object: `cursor`
- official-doc query: `cursor install official documentation`
- general fallback query: `how to install cursor`

For install tasks the worker now runs the official-documentation web-search
query before wikiHow. If that does not produce ranked guidance, it continues to
the existing wikiHow/community fallback and recursive explicit-step source gate.

## Verification

- Added Rust regression coverage in
  `tests/unit/specification/reasoning_paths_procedures.rs` for English,
  Russian, Hindi, and Chinese install prompts.
- Added `tests/e2e/tests/issue-501.spec.js` for the browser/WASM worker path.
