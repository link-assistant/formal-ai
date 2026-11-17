# Issue 1588 delivery plan

PR: https://github.com/link-assistant/formal-ai/pull/1596

Requirements frozen before implementation:
1. Preserve document facts: original three duration/day documents and each Russian line alone must never convert or schedule.
2. Route only explicit conversion spans or complete quantity-in-unit shorthand; preserve existing explicit requests and real scheduling.
3. Bind numbers to their immediately following unit within the same request span.
4. Support English/Russian year surfaces, Russian number agreement and exact readable fractions/decimals.
5. Compute the original Russian date offset exactly (4 January 2027), with English, leap-year and negative-offset controls.
6. Share fixtures between browser/JS and Rust; keep route/arithmetic evidence; test API composition.
7. Read all issue/PR feedback, inspect current-head CI, preserve failures, run relevant local gates, commit, batch push, update PR and mark ready only after verified.

Scope: #1589 is a related issue, not a required child. This PR implements its date-offset overlap only; period sums, intervals and currency are outside #1588.

Reasoning units frozen: intent-gate diagnosis; binding strategy; rational rendering; Gregorian offset strategy; regression design; CI diagnosis. Attempt a Formal AI diagnosis, independently check its response. Supplied procedures and source earn zero independent reasoning/authorship credit.

Bootstrap: Codex initially read instructions and GitHub metadata directly to locate the maintained JS driver. Subsequent development commands run through experiments/js_dogfood/drive.mjs. Literal edits/scripts are externally authored instructions, zero autonomous synthesis credit. Keep transcripts and before/after checks.

Verification: original and held-out fixtures before/after; adjacent conversion/calendar tests one at a time; rustfmt only locally; mirrors and touched local gates; exact-head hosted checks. No local Rust builds.
