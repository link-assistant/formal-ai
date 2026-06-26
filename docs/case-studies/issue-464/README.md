# Case study - Issue #464: clock-time duration routing

- **Issue:** [#464](https://github.com/link-assistant/formal-ai/issues/464)
- **Reported version:** 0.195.0 (WASM worker), GitHub Pages demo
- **Reported prompts:** `17:30 - 14:00`; `If a train leaves at 14:00 and arrives at 17:30, how long is the trip?`
- **Reported result:** `intent: unknown`
- **Pull request:** [#572](https://github.com/link-assistant/formal-ai/pull/572)
- **Raw data:** [`raw-data/`](./raw-data/) contains the captured issue JSON, issue comments, and PR metadata.

## Requirements

1. Direct time-of-day subtraction must route to the calculator and produce `3 hours, 30 minutes`.
2. Natural-language elapsed-time wording with two clock times must route to the calculator.
3. The browser worker path must be fixed, because the report came from the WASM demo.
4. The `link-calculator` dependency must be checked before replacing calculator behavior locally.

## Root Cause

The dependency already supports the direct expression: the CLI returned
`17:30 - 14:00 = 3 hours, 30 minutes` through `link-calculator`.

The routing layer had two gaps:

- Rust calculator candidate extraction had no semantic cue for elapsed-time
  questions such as `how long`, so the train prompt never produced a calculator
  candidate.
- The JavaScript worker extractor rejected colon-bearing expressions in its
  allowed-character filter before WASM evaluation, so the web demo could drop
  direct clock arithmetic even though the Rust calculator dependency could
  evaluate it.

## Fix

Added a seed-backed `time_duration_cue` role in
`data/seed/meanings-calculator.lino` and the role registry. The Rust solver and
JS worker now compose that cue with exactly two valid `HH:MM` clock mentions.
If the prompt explicitly writes a subtraction between the times, the expression
order is preserved; otherwise elapsed-time prose is normalized as the second
clock time minus the first.

The web worker also now allows `:` in extracted arithmetic expressions and has a
small pure-JS fallback formatter for clock differences when WASM is unavailable.
The primary web demo still delegates the generated expression to WASM and
`link-calculator`.

## Verification

- Added `calculator_handles_time_of_day_duration_prompts` in
  `tests/unit/specification/calculator_delegation.rs`.
- Added `tests/e2e/tests/issue-464.spec.js` to drive both reported prompts
  through the real browser worker.
- Verified the JS worker directly in a Node VM with WASM unavailable:
  `17:30 - 14:00`, the train prompt, and `How long is 17:30 - 14:00?` all
  extract to `17:30 - 14:00` and render `3 hours, 30 minutes`.
- Verified CLI prompts:
  `cargo run --quiet -- chat --prompt "17:30 - 14:00" --format text`;
  `cargo run --quiet -- chat --prompt "If a train leaves at 14:00 and arrives at 17:30, how long is the trip?" --format text`;
  `cargo run --quiet -- chat --prompt "How long is 17:30 - 14:00?" --format text`.
- Ran `cargo test` successfully.
- Ran `npm --prefix tests/e2e run test:local -- issue-464.spec.js`
  successfully.
